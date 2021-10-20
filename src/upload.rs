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
        return (Status::BadRequest, Some("missing file name"));
    }

    let filename = file.name().unwrap().to_string();

    if contains_unsafe_chars(&filename) {
        return (Status::BadRequest, Some("invalid file name"));
    }

    if filename.len() > 64 {
        return (Status::BadRequest, Some("too long file name"));
    }

    file.persist_to(format!("storage/uploads/{}", &filename)).await.expect("file not saved");

    (Status::Ok, Some("ok"))
}