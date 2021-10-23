use std::borrow::Borrow;
use std::ffi::OsStr;
use std::fmt::{Debug};
use std::ops::Sub;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use rocket::{Request};
use rocket::form::{Form};
use rocket::fs::{FileName, NamedFile};
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::serde::json::{json, Value};
use serde::Serialize;

use crate::AppState;
use crate::auth::Token;
use crate::user::User;

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

impl FileScope {
    pub fn get_path_to_common_folder() -> PathBuf {
        let mut path = PathBuf::from("storage");

        path.push("uploads");
        path.push("common");

        path
    }

    pub fn get_path_to_common_file(filename: impl AsRef<OsStr>) -> PathBuf {
        let mut path = Self::get_path_to_common_folder();

        path.push(filename.as_ref().to_str().expect("invalid filename").to_string());

        path
    }

    pub fn get_path_to_file(&self, filename: impl AsRef<OsStr>, user: Option<Arc<User>>) -> PathBuf {
        match self {
            Self::Common => Self::get_path_to_common_file(filename),
            Self::User => user.unwrap().get_path_to_user_file(filename),
        }
    }
}

pub struct UserToken {
    pub(crate) user: Arc<User>,
    pub(crate) token: Token,
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
        let mut tokens = app_state.tokens.write().await;

        let user_token = tokens.list.get_key_value(token);

        if user_token.is_none() {
            return Outcome::Failure((Status::Unauthorized, "invalid token"));
        }

        let token_time = tokens.lifespans.get(token)
            .map_or_else(
                || Instant::now().sub(Duration::from_secs(1)),
                |x| x.to_owned(),
            );

        if token_time <= Instant::now() {
            tokens.list.remove(token);
            tokens.lifespans.remove(token);

            return Outcome::Failure((Status::Unauthorized, "expired token"));
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
    const MAX_FILES: u64 = 10;
    let cursor = cursor.unwrap_or(0);

    let path = {
        let mut path = PathBuf::from("storage");

        path.push("uploads");

        match scope.unwrap_or_default() {
            FileScope::User => {
                path.push("user");
                path.push(ut.user.username().to_string());
            }
            FileScope::Common => path.push("common"),
        }

        path
    };

    let rdir = tokio::fs::read_dir(&path).await;

    if rdir.is_err() {
        log::warn!("failed to read_dir {:?}: {:?}", &path, rdir.unwrap_err());

        return Ok(json!({}));
    }

    let (files, more) = {
        let mut offset = cursor;

        let mut rdir = rdir.unwrap();

        let mut files: Vec<File> = vec![];

        let mut current_files = 0u64;
        let mut more = false;

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

            if !entry.file_type().await.map_or_else(|_| false, |x| x.is_file()) {
                continue;
            }

            let filename = entry.file_name();
            let filename = filename.to_str().expect("filename with invalid chars").to_owned();

            if filename.starts_with('.') {
                continue;
            }

            if current_files >= MAX_FILES {
                more = true;
                break;
            }

            if offset > 0 {
                offset -= 1;
                continue;
            }

            current_files += 1;

            files.push(File {
                name: filename,
            })
        }

        (files, more)
    };


    Ok(json!({
        "files": files,
        "nextCursor": more.then(|| cursor + MAX_FILES),
        "prevCursor": cursor.checked_sub(MAX_FILES),
    }))
}

#[derive(FromForm, Debug)]
pub struct DownloadData<'r> {
    filename: &'r str,
    scope: FileScope,
}

#[post("/files/download", data = "<form>")]
pub async fn download_file(ut: UserToken, form: Form<DownloadData<'_>>) -> Result<Option<NamedFile>, Status> {
    {
        let filename = form.filename.split_once('.').map_or_else(|| form.filename, |(x, _)| x);
        if !FileName::new(filename).is_safe() {
            log::debug!("illegal chars detected in filename");

            return Err(Status::BadRequest);
        }
    }

    let path = form.scope.get_path_to_file(form.filename, Some(ut.user.clone()));

    let file = NamedFile::open(&path).await;

    if file.is_err() {
        log::debug!("tried to download non-existent file {:?}", path);

        return Ok(None);
    }

    Ok(file.ok())
}