use std::fs;
use std::fs::FileType;
use std::io::{Read, Write};

use rocket::form::FromForm;
use rocket::form::validate::Contains;
use rocket::http::ext::IntoCollection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize, Serialize)]
pub struct User {
    username: Arc<str>,
    password: Option<String>,

    file_prefixes: Vec<Arc<str>>,

    hashed_password: Option<String>,
}

impl User {
    pub fn username(&self) -> Arc<str> {
        self.username.clone()
    }

    pub fn check_password(&self, unknown: &str) -> bool {
        use argon2::{Argon2, password_hash::{PasswordVerifier, PasswordHash}};

        if self.hashed_password.is_none() {
            return false;
        }

        let argon2 = Argon2::default();
        let parsed_hash = {
            let hashed = self.hashed_password.as_ref().unwrap();

            PasswordHash::new(hashed.as_str()).unwrap()
        };

        argon2.verify_password(unknown.as_bytes(), &parsed_hash).is_ok()
    }

    pub fn prefixes(&self) -> &Vec<Arc<str>> {
        &self.file_prefixes
    }

    pub fn hash_password(&mut self) -> bool {
        use argon2::{
            password_hash::{
                rand_core::OsRng,
                PasswordHash, PasswordHasher, SaltString,
            },
            Argon2,
        };

        if self.password.is_none() {
            return false;
        }

        let salt = SaltString::generate(&mut OsRng);

        let argon2 = Argon2::default();

        let password_hash = {
            let plaintxt = self.password.as_ref().unwrap();

            argon2.hash_password(plaintxt.as_bytes(), &salt).unwrap().to_string()
        };

        self.hashed_password = Some(password_hash);
        self.password = None;

        true
    }
}

pub fn get_users() -> Vec<User> {
    fs::read_dir("storage/users")
        .expect("couldn't exec storage/user folder")
        .filter_map(|maybe_file| {
            if let Ok(file) = maybe_file {
                if !file.file_type().unwrap().is_file() {
                    log::debug!("skipped not a real file: {:?}", file.path());
                    return None;
                }

                if let None = file.file_name().to_str() {
                    log::warn!("error while transmuting file name to string");
                    return None;
                }

                let file_name = {
                    let os_name = file.file_name();
                    let name = os_name.to_str();

                    name.unwrap().to_string()
                };

                if !file_name.starts_with('.') && file_name.ends_with(".toml") {
                    return Some(file);
                }
            }

            log::debug!("user dir entry invalid, skipping");

            None
        })
        .filter_map(|file| {
            let file_contents = fs::read_to_string(file.path());

            if let Err(why) = &file_contents {
                log::warn!("couldn't read user file data ({:?}): {}", file.path(), why);
                return None;
            }

            let data = {
                let file_contents = file_contents.unwrap();

                toml::from_str::<User>(&file_contents)
            };

            if let Err(why) = &data {
                log::warn!("invalid user file schema ({:?}): {}", file.path(), why);
                return None;
            }

            let (data, hashed_recently) = {
                let mut data = data.unwrap();

                let hashed_recently = data.hash_password();

                (data, hashed_recently)
            };

            if hashed_recently {
                let serialized_data = toml::to_string(&data).unwrap();
                log::info!("writing hashed password to {:?}", file.path());
                fs::write(file.path(), serialized_data);
            } else {
                log::debug!("skipped already hashed password in file {:?}", file.path());
            }

            Some(data)
        })
        .collect()
}