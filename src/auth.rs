use std::error::Error;

use rocket::{Data, State};
use rocket::data::Capped;
use rocket::form::{DataField, Form, Strict};
use rocket::fs::TempFile;
use rocket::http::Status;

use crate::AppState;

#[derive(FromForm, Debug)]
pub struct LoginData<'r> {
    user: &'r str,
    pass: &'r str,
}

pub type Token<'t> = &'t str;

#[post("/login", data = "<form>")]
pub async fn login(mut form: Form<LoginData<'_>>, state: &State<AppState<'_>>) -> Status {
    Status::Ok
}