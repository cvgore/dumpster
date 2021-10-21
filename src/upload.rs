use std::error::Error;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use rocket::{Data, State};
use rocket::data::Capped;
use rocket::form::{DataField, Form, Strict};
use rocket::fs::TempFile;
use rocket::http::Status;
use rocket::serde::json::{json, Value};

use crate::AppState;

#[derive(FromForm, Debug)]
pub struct UploadData<'r> {
    file: TempFile<'r>,
}

const UNSAFE_CHARS: [&str; 11] = ["//", "\\", "..", "<", ">", ":", "\"", "|", "?", "*", "\0"];

fn contains_unsafe_chars(name: &str) -> bool {
    for ch in UNSAFE_CHARS {
        if name.contains(ch) {
            return true;
        }
    }

    false
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
        let mut path = PathBuf::from("storage");

        path.push("uploads");
        path.push("common");
        path.push(format!("{}-{}.{}", &filename, ts.as_secs(), &ext));

        path
    };

    log::debug!("will store file @ {:?}", path);

    if let Err(why) = file.persist_to(&path).await {
        log::warn!("uploaded file store error: {}", why);
    }

    Ok(())
}