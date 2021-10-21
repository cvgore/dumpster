use std::error::Error;

use rocket::{Data, State};
use rocket::data::Capped;
use rocket::form::{DataField, Form, Strict};
use rocket::fs::TempFile;
use rocket::http::Status;

use crate::AppState;

#[derive(FromForm, Debug)]
pub struct UploadData<'r> {
    file: TempFile<'r>
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
pub async fn upload(mut form: Form<UploadData<'_>>, state: &State<AppState<'_>>) -> (Status, Option<&'static str>) {
    let file = &mut form.file;

    if file.name().is_none() {
        return (Status::BadRequest, Some("missing or invalid file name"));
    }

    if file.content_type().is_none() {
        return (Status::BadRequest, Some("missing content type"));
    }

    let filename = file.name().unwrap().to_string();

    if filename.len() > 64 {
        return (Status::BadRequest, Some("too long file name"));
    }

    let content_type = file.content_type().unwrap().extension();

    if content_type.extension().is_none() {
        return (Status::BadRequest, Some("cannot infer extension from content type"));
    }

    let ext = content_type.extension().unwrap();

    if ext.len() > 16 {
        return (Status::BadRequest, Some("too long extension"));
    }

    file.persist_to(format!("storage/uploads/{}-{}.{}", &filename, &ext)).await;

    (Status::Ok, Some("ok"))
}