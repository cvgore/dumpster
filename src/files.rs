use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Write};
use std::sync::Arc;

use rand::{Rng, RngCore};
use rocket::{Data, Request, State};
use rocket::data::Capped;
use rocket::form::{DataField, Form, Strict};
use rocket::fs::{TempFile, NamedFile};
use rocket::http::ext::IntoCollection;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::serde::json::{json, Value};
use serde::Serialize;

use crate::AppState;
use crate::auth::Token;
use crate::user::User;
use std::path::PathBuf;
use std::borrow::Borrow;

#[derive(Debug, PartialEq, FromFormField)]
pub enum FileScope {
    Common,
    User,
}

impl Default for FileScope {
    fn default() -> Self {
        FileScope::Common
    }
}

pub struct UserToken {
    user: Arc<User>,
    token: Token,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserToken {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let token_recv = request.headers().get_one("Authorization");
        if token_recv.borrow().is_none() {
            return Outcome::Failure((Status::Unauthorized, "token missing"));
        }

        let token = token_recv.unwrap();
        if !token.starts_with("Bearer ") || token.len() < 8 {
            return Outcome::Failure((Status::Unauthorized, "invalid token type"));
        }

        let (_, token) = token.split_at(7);

        let state = request.rocket().state::<AppState>();

        if state.borrow().is_none() {
            return Outcome::Failure((Status::Unauthorized, "state missing"));
        }

        let app_state = state.unwrap();
        let tokens = app_state.tokens.read().await;

        let user_token = tokens.get_key_value(token);

        if user_token.is_none() {
            return Outcome::Failure((Status::Unauthorized, "invalid token"));
        }

        let (token, user) = user_token.unwrap();

        Outcome::Success(UserToken {
            user: user.clone(),
            token: token.clone(),
        })
    }
}

#[derive(Serialize)]
struct File {
    name: String,
}

#[get("/files?<scope>&<cursor>")]
pub async fn list(ut: UserToken, scope: Option<FileScope>, cursor: Option<u64>) -> Result<Value, Status> {
    let path = {
        let mut path = PathBuf::from("storage");

        path.push("uploads");

        match scope.unwrap_or(Default::default()) {
            FileScope::User => {
                path.push("user");
                path.push(ut.user.username().clone().to_string());
            },
            FileScope::Common => path.push("common"),
        }

        path
    };

    let rdir = tokio::fs::read_dir(&path).await;

    if rdir.is_err() {
        log::warn!("failed to read_dir {:?}: {:?}", &path, rdir.unwrap_err());

        return Ok(json!({}));
    }

    let files = {
        let mut rdir = rdir.unwrap();

        let mut files: Vec<File> = vec![];

        loop {
            let entry = rdir.next_entry().await;

            if entry.borrow().is_err() {
                log::warn!("io error while reading dir {:?} {:?}", &path, entry.unwrap_err());

                continue;
            }

            let entry = entry.unwrap();

            if entry.borrow().is_none() {
                break;
            }

            let entry = entry.unwrap();
            let filename = entry.file_name();
            let filename = filename.to_str().expect("filename with invalid chars").to_owned();

            if filename.starts_with(".") {
                continue;
            }

            files.push(File {
                name: filename.into(),
            })
        }

        files
    };

    Ok(json!({
        "files": files
    }))
}

#[derive(FromForm, Debug)]
pub struct DownloadData<'r> {
    filename: &'r str,
    scope: FileScope,
}


#[post("/files/download", data = "<form>")]
pub async fn download_file(ut: UserToken, form: Form<DownloadData<'_>>) -> Option<NamedFile> {
    let path = {
        let mut path = PathBuf::from("storage");

        path.push("uploads");

        match &form.scope {
            FileScope::User => {
                path.push("user");
                path.push(ut.user.username().clone().to_string());
            },
            FileScope::Common => path.push("common"),
        }

        path.push(form.filename);

        path
    };


    let file = NamedFile::open(&path).await;

    if file.is_err() {
        log::debug!("tried to download non-existent file {:?}", path);

        return None;
    }

    file.ok()
}