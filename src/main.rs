#[macro_use]
extern crate rocket;

use std::collections::HashMap;
use std::sync::Arc;

use rocket::data::{Limits, ToByteUnit};
use rocket::fs::FileServer;
use rocket::fs::relative;
use rocket::futures::StreamExt;
use tokio::sync::RwLock;

use crate::auth::Token;
use crate::user::{get_users, User};
use std::rc::Rc;

mod upload;
mod user;
mod auth;

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

#[catch(400)]
fn bad_request() -> &'static str {
    "🍆 400 Fucked-up Request"
}

pub struct AppState<'s> {
    users: Vec<Arc<User>>,
    prefix_map: HashMap<Arc<str>, Arc<User>>,
    tokens: RwLock<HashMap<Token<'s>, Arc<User>>>,
    user_files_counter: RwLock<HashMap<AtomicU64>>
}

impl<'s> AppState<'s> {
    pub fn new_from_users() -> Self {
        let users = get_users()
            .into_iter()
            .map(|x| Arc::new(x))
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
        .mount("/ajax", routes![upload::upload, auth::login])
        .register("/", catchers![not_found, payload_too_large, unprocessable_entity, bad_request])
        .mount("/", FileServer::from(relative!("public")))
}