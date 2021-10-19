use std::error::Error;

use rocket::{Data, State};
use rocket::data::Capped;
use rocket::form::{DataField, Form, Strict};
use rocket::fs::TempFile;
use rocket::http::Status;

use crate::AppState;

#[derive(FromForm)]
pub struct UploadData<'r> {
    #[field(validate = range(1..))]
    #[field(name = "flowChunkNumber")]
    flow_chunk_number: u16,

    #[field(validate = range(1..))]
    #[field(name = "flowChunkSize")]
    flow_chunk_size: u32,

    #[field(validate = range(1..))]
    #[field(name = "flowCurrentChunkSize")]
    flow_current_chunk_size: u32,

    #[field(validate = range(1..))]
    #[field(name = "flowTotalSize")]
    flow_total_size: u64,

    #[field(validate = len(1..))]
    #[field(name = "flowIdentifier")]
    flow_identifier: &'r str,

    #[field(validate = len(1..))]
    #[field(validate = omits(['.']))]
    #[field(name = "flowFilename")]
    flow_filename: &'r str,

    #[field(validate = len(1..))]
    #[field(name = "flowRelativePath")]
    flow_relative_path: &'r str,

    #[field(validate = range(1..))]
    #[field(name = "flowTotalChunks")]
    flow_total_chunks: u16,

    file: TempFile<'r>,
}

#[post("/", data = "<upload_data>")]
pub fn upload(form: Form<Strict<UploadData<'_>>>, state: &State<AppState>) -> Status {
    if !form.flow_filename.contains(["//", "\\", "..", "<", ">", ":", "\"", "|", "?", "*", "\0"]) {
        Status::BadRequest("illegal chars in filename")
    }

    if form.flow_filename.len() > 60 {
        Status::BadRequest("filename too long")
    }

    form.file.persist_to(format!("storage/upload_chunks/{}.{}.part", form.flow_filename, form.flow_chunk_number));

    Status::Ok
}