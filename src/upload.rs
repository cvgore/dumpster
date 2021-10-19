use std::error::Error;

use rocket::{Data, State};
use rocket::data::Capped;
use rocket::form::{DataField, Form, Strict};
use rocket::fs::TempFile;
use rocket::http::Status;

use crate::AppState;

#[derive(FromForm)]
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

#[post("/", data = "<form>")]
pub async fn upload(mut form: Form<UploadData<'_>>, state: &State<AppState>) -> Status {
    let file = &mut form.file;

    if file.name().is_none() {
        return Status::BadRequest;
    }

    let filename = file.name().unwrap().to_string();

    if contains_unsafe_chars(&filename) {
        return Status::BadRequest;
    }

    if filename.len() > 64 {
        return Status::BadRequest;
    }

    file.persist_to(format!("storage/upload/{}", &filename));

    Status::Ok
}