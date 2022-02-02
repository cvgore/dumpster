use std::collections::HashMap;
use std::ffi::OsStr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use rocket::State;
use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::http::Status;
use rocket::serde::json::{json, Value};
use rocket::serde::json::serde_json::json;

use crate::AppState;
use crate::files::FileScope;
use crate::user::User;

#[derive(FromForm, Debug)]
pub struct UploadData<'r> {
    file: TempFile<'r>,
}

fn sanitize_filename<'r>(given_filename: impl AsRef<OsStr>) -> Option<String> {
    #[cfg(not(unix))]
        let (bad_char, bad_name) = {
        static BAD_CHARS: &[char] = &[
            // Microsoft says these are invalid.
            '.', '<', '>', ':', '"', '/', '\\', '|', '?', '*',

            // `cmd.exe` treats these specially.
            ',', ';', '=',

            // These are treated specially by unix-like shells.
            '(', ')', '&', '#',
        ];

        // Microsoft says these are reserved.
        static BAD_NAMES: &[&str] = &[
            "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4",
            "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2",
            "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
        ];

        let bad_char = |c| BAD_CHARS.contains(&c) || c.is_control();
        let bad_name = |n| BAD_NAMES.contains(&n);
        (bad_char, bad_name)
    };

    #[cfg(unix)]
        let (bad_char, bad_name) = {
        static BAD_CHARS: &[char] = &[
            // These have special meaning in a file name.
            '.', '/', '\\',

            // These are treated specially by shells.
            '<', '>', '|', ':', '(', ')', '&', ';', '#', '?', '*',
        ];

        let bad_char = |c| BAD_CHARS.contains(&c) || c.is_control();
        let bad_name = |_| false;
        (bad_char, bad_name)
    };

    // Get the file name as a `str` without any extension(s).
    let file_name = std::path::Path::new(&given_filename)
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| n.split(bad_char).filter(|s| !s.is_empty()).next())?;

    // At this point, `file_name` can't contain `bad_chars` because of
    // `.split()`, but it can be empty or reserved.
    if file_name.is_empty() || bad_name(file_name) {
        return None;
    }

    let extension = std::path::Path::new(&given_filename)
        .extension()
        .and_then(|x| x.to_str())
        .and_then(|x| x.split(bad_char).filter(|s| !s.is_empty()).next())?;

    // At this point, `file_name` can't contain `bad_chars` because of
    // `.split()`, but it can be empty or reserved.
    if extension.is_empty() || bad_name(extension) {
        return Some(file_name.into());
    }

    Some(format!("{}.{}", file_name, extension))
}

fn guess_scope_from_filename(filename: impl AsRef<OsStr>, prefixes: &HashMap<Arc<str>, Arc<User>>) -> (FileScope, Option<Arc<User>>) {
    let filename = filename.as_ref().to_str().expect("invalid filename");

    let split = filename.split_once('_');

    log::debug!("splitted filename: {:?}, got {:?}", filename, &split);

    if split.as_ref().is_none() {
        return (FileScope::Common, None);
    }

    let (prefix, _) = split.unwrap();

    log::debug!("got prefix {}", prefix);

    if prefix.is_empty() {
        return (FileScope::Common, None);
    }

    log::debug!("prefix list {:?}", prefixes.keys());

    let prefix = format!("{}_", prefix);

    prefixes.get(prefix.as_str()).map_or_else(
        || (FileScope::Common, None),
        |x| (FileScope::User, Some(x.clone())),
    )
}

#[post("/upload", data = "<form>")]
pub async fn upload(mut form: Form<UploadData<'_>>, state: &State<AppState>) -> Result<(), (Status, Value)> {
    let file = &mut form.file;

    if file.name().is_none() {
        log::warn!("tried to upload file with invalid or missing filename");

        return Err((Status::BadRequest, json!({
            "error": "invalid filename"
        })));
    }

    let filename = file.raw_name()
        .map(|x| x.dangerous_unsafe_unsanitized_raw())
        .map(|x| x.to_string());

    if filename.as_ref().is_none() {
        log::info!("tried to upload file without filename or invalid filename");

        return Err((Status::BadRequest, json!({
            "error": "filename empty or invalid"
        })));
    }

    let filename = filename.unwrap();

    if filename.len() > 64 {
        log::info!("tried to upload file with too long filename");

        return Err((Status::BadRequest, json!({
            "error": "filename too long"
        })));
    }

    let filename = sanitize_filename(filename);

    if filename.as_ref().is_none() {
        log::warn!("tried to upload file with invalid filename chars");

        return Err((Status::BadRequest, json!({
            "error": "invalid filename"
        })));
    }

    let filename = filename.unwrap();

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards");

    let path = {
        let (scope, user) = guess_scope_from_filename(
            &filename, &state.prefix_map,
        );

        let filename = format!("{}-{}", ts.as_millis(), filename);
        scope.get_path_to_file(&filename, user)
    };


    log::debug!("will store file @ {:?}", path);

    if let Err(why) = file.persist_to(&path).await {
        log::warn!("uploaded file store error: {}", why);
    }

    Ok(())
}