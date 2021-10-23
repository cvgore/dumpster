use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use rocket::{Data, State};
use rocket::data::Capped;
use rocket::form::{DataField, Form, Strict};
use rocket::fs::TempFile;
use rocket::http::Status;
use rocket::serde::json::{json, Value};

use crate::AppState;
use crate::files::FileScope;
use crate::user::User;

#[derive(FromForm, Debug)]
pub struct UploadData<'r> {
    file: TempFile<'r>,
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

    if prefix.len() == 0 {
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

    let filename = file.name().unwrap().to_string();

    if filename.len() > 64 {
        log::info!("tried to upload file with too long filename");

        return Err((Status::BadRequest, json!({
            "error": "filename too long"
        })));
    }

    if file.content_type().is_none() {
        log::info!("tried to upload file without content type");

        return Err((Status::BadRequest, json!({
            "error": "missing content type"
        })));
    }

    let content_type = file.content_type().unwrap();

    if content_type.extension().is_none() {
        log::info!("tried to upload file with unknown extension or content type");

        return Err((Status::BadRequest, json!({
            "error": "unknown content type"
        })));
    }

    let ext = content_type.extension().unwrap();

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards");

    let path = {
        let (scope, user) = guess_scope_from_filename(
            &filename, &state.prefix_map,
        );

        let filename = format!("{}-{}", filename, ts.as_millis());
        let path = scope.get_path_to_file(&filename, user);

        path.with_extension(ext.to_string())
    };


    log::debug!("will store file @ {:?}", path);

    if let Err(why) = file.persist_to(&path).await {
        log::warn!("uploaded file store error: {}", why);
    }

    Ok(())
}