use std::error::Error;

use rocket::data::Capped;
use rocket::form::{DataField, Form, Strict};
use rocket::fs::TempFile;
use rocket::Data;
use rocket::http::Status;

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
pub fn upload(upload_data: Form<Strict<UploadData<'_>>>) -> Status {
    if upload_data.file.persist_to().await {

    }

    Status::Ok
}