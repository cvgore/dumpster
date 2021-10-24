#![deny(warnings)]
#![deny(clippy::all)]
#[macro_use]
extern crate rocket;

use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::time::Instant;

use rocket::fs::FileServer;
use tokio::sync::RwLock;

use crate::auth::Token;
use crate::user::{get_users, User};

mod upload;
mod user;
mod auth;
mod files;

#[catch(404)]
fn not_found() -> &'static str {
    "🍆 404 Fucked Out"
}

#[catch(413)]
fn payload_too_large() -> &'static str {
    "🍆 413 Request Too Fucking"
}

#[catch(422)]
fn unprocessable_entity() -> &'static str {
    "🍆 422 Infucking Request"
}

#[catch(401)]
fn unauthorized() -> &'static str {
    "🍆 401 Unfucktorized"
}

#[catch(400)]
fn bad_request() -> &'static str {
    "🍆 400 Fucked-up Request"
}

#[catch(429)]
fn too_many_requests() -> &'static str {
    "🍆 429 Go Fuck Yourself"
}

#[catch(500)]
fn internal_server_error() -> &'static str {
    "🍆 500 Internal Server Fucking"
}

#[derive(Default)]
struct TokensVec {
    list: HashMap<Token, Arc<User>>,
    lifespans: HashMap<Token, Instant>,
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
            .map(|user| {
                let _ = fs::read_dir(user.get_path_to_user_folder())
                    .map_err(|_| {
                        fs::create_dir(user.get_path_to_user_folder())
                            .expect("failed to create user folder");
                    });

                Arc::new(user)
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
                hm.insert(user.username(), user.clone());

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
        .mount("/ajax", routes![
            upload::upload,
            auth::login,
            files::list,
            files::download_file,
            auth::logout
        ])
        .register("/", catchers![
            not_found,
            payload_too_large,
            unprocessable_entity,
            bad_request,
            unauthorized,
            internal_server_error,
            too_many_requests
        ])
        .mount("/", FileServer::from("public"))
}