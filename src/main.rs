#[macro_use]
extern crate rocket;

use rocket::data::{Limits, ToByteUnit};
use rocket::fs::FileServer;
use rocket::fs::relative;

use crate::auth::{get_users, User};

mod upload;
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
    "🍆 400 Fcked-up Request"
}

pub struct AppState {
    users: Vec<User>,
}

#[launch]
fn rocket() -> _ {
    env_logger::init();

    rocket::build()
        .manage(AppState {
            users: get_users()
        })
        .mount("/upload", routes![upload::upload])
        .register("/", catchers![not_found, payload_too_large, unprocessable_entity, bad_request])
        .mount("/", FileServer::from(relative!("public")))
}