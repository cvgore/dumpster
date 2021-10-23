#[macro_use]
extern crate rocket;

use std::collections::{HashMap, HashSet};
use std::fs;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::Instant;

use rocket::data::{Limits, ToByteUnit};
use rocket::fs::FileServer;
use rocket::fs::relative;
use rocket::futures::StreamExt;
use tokio::sync::RwLock;

use crate::auth::Token;
use crate::user::{get_users, User};

mod upload;
mod user;
mod auth;
mod files;

#[catch(404)]
fn not_found() -> &'static str {
    "🍆 404 Not Found"
}

#[catch(413)]
fn payload_too_large() -> &'static str {
    "🍆 413 Request Too Heavy"
}

#[catch(422)]
fn unprocessable_entity() -> &'static str {
    "🍆 422 Invalid Request"
}

#[catch(401)]
fn unauthorized() -> &'static str {
    "🍆 401 Unauthorized"
}

#[catch(400)]
fn bad_request() -> &'static str {
    "🍆 400 Fucked-up Request"
}

#[catch(500)]
fn internal_server_error() -> &'static str {
    "🍆 500 Internal Server Error"
}

struct TokensVec {
    list: HashMap<Token, Arc<User>>,
    lifespans: HashMap<Token, Instant>,
}

impl Default for TokensVec {
    fn default() -> Self {
        Self {
            list: Default::default(),
            lifespans: Default::default(),
        }
    }
}

pub struct AppState {
    users: HashMap<Arc<str>, Arc<User>>,
    prefix_map: HashMap<Arc<str>, Arc<User>>,
    tokens: RwLock<TokensVec>,
}

impl AppState {
    pub fn new_from_users() -> Self {
        let users = get_users()
            .into_iter()
            .map(|x| {
                let dir = fs::read_dir(x.get_path_to_user_folder());

                if let Err(_) = dir {
                    fs::create_dir(x.get_path_to_user_folder());
                }

                Arc::new(x)
            })
            .collect::<Vec<Arc<User>>>();

        let prefix_map = users
            .clone()
            .into_iter()
            .fold(HashMap::with_capacity(users.len()), |mut hm, user| {
                for prefix in user.prefixes() {
                    let p = prefix.clone();
                    hm.insert(p.clone(), user.clone());
                }

                hm
            });

        let users = users
            .clone()
            .into_iter()
            .fold(HashMap::with_capacity(users.len()), |mut hm, user| {
                hm.insert(user.username().clone(), user.clone());

                hm
            });

        Self {
            users,
            prefix_map,
            tokens: Default::default(),
        }
    }
}

#[launch]
fn rocket() -> _ {
    env_logger::init();

    rocket::build()
        .manage(AppState::new_from_users())
        .mount("/ajax", routes![upload::upload, auth::login, files::list, files::download_file])
        .register("/", catchers![
            not_found, payload_too_large, unprocessable_entity, bad_request, unauthorized, internal_server_error
        ])
        .mount("/", FileServer::from(relative!("public")))
}