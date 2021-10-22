use std::error::Error;
use std::fmt;
use std::fmt::Write;
use std::sync::Arc;

use rand::{Rng, RngCore};
use rocket::{Data, State};
use rocket::data::Capped;
use rocket::form::{DataField, Form, Strict};
use rocket::fs::TempFile;
use rocket::http::Status;
use rocket::serde::json::{json, Value};

use crate::AppState;
use crate::files::UserToken;

#[derive(FromForm, Debug)]
pub struct LoginData<'r> {
    user: &'r str,
    pass: &'r str,
}

pub type Token = Arc<str>;

fn new_token() -> String {
    const TOKEN_BYTES: usize = 32;

    let token_bytes = {
        let mut buf = [0u8; TOKEN_BYTES];

        rand::thread_rng().fill_bytes(&mut buf);

        buf
    };

    let token = {
        let mut s = String::with_capacity(TOKEN_BYTES);

        for byte in token_bytes {
            write!(&mut s, "{:x}", byte).expect("stringification of token failed");
        }

        s
    };

    token
}

#[post("/login", data = "<form>")]
pub async fn login(mut form: Form<LoginData<'_>>, state: &State<AppState>) -> Result<Value, Status> {
    let entry = state.users.get(form.user);

    if entry.is_none() {
        log::debug!("user '{}' not found", form.user);

        return Err(Status::Unauthorized);
    }

    let user = entry.unwrap().clone();

    if !user.check_password(form.pass) {
        log::debug!("password for user '{}' mismatch", form.user);

        return Err(Status::Unauthorized);
    }

    let mut tokens = state.tokens.write().await;

    let token = new_token();

    tokens.insert(token.clone().into_boxed_str().into(), user);

    Ok(json!({
        "token": token
    }))
}

#[post("/logout")]
pub async fn logout(ut: UserToken, state: &State<AppState>) -> Status {
    let mut tokens = state.tokens.write().await;

    tokens.remove(&ut.token);

    Status::Ok
}