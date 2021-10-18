#[macro_use]
extern crate rocket;

use crate::config::Config;
use rocket::data::{Limits, ToByteUnit};
use rocket::fs::FileServer;
use rocket::fs::relative;
use crate::auth::{get_users, User};

mod upload;
mod auth;
mod config;

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

struct State<'s> {
    users: Vec<User<'s>>
}

#[launch]
fn rocket() -> _ {
    env_logger::init();

    let rocket = rocket::build()
        .mount("/upload", routes![upload::upload])
        .register("/", catchers![not_found, payload_too_large, unprocessable_entity])
        .mount("/", FileServer::from(relative!("public")));

    let users = get_users(&rocket.figment().extract::<Config>().expect("missing app_secret"));

    rocket.manage(State {
        users
    });

    rocket
}