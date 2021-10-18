use serde::Deserialize;
use std::fs;
use rocket::http::ext::IntoCollection;
use rocket::form::validate::Contains;
use std::fs::FileType;
use std::io::{Write, Read};
use crypto::sha2::Digest;
use crypto::sha2::digest::generic_array::functional::FunctionalSequence;
use crypto::sha2::digest::DynDigest;
use crypto::sha2;
use crate::config::Config;

#[derive(Deserialize)]
pub struct User<'u> {
    username: &'u str,
    password: Option<&'u str>,
    hashed_password: Option<&'u str>,

    prefixes: Vec<&'u str>,
}

impl<'u> User {
    pub fn username(&self) -> &'u str {
        self.username
    }

    pub fn check_password(&self, unknown: impl ToString) -> bool {
        if self.hashed_password.is_none() {
            return false;
        }

        let known_string = self.hashed_password.unwrap().as_bytes();

        crypto::util::fixed_time_eq(
            known_string, unknown.as_bytes()
        )
    }

    pub fn prefixes(&self) -> &Vec<&'u str> {
        &self.prefixes
    }

    pub fn hash_password(&mut self, salt: impl ToString) {
        if self.password.is_none() {
            return;
        }

        let data = {
            let mut digest = crypto::sha2::Sha256::new();
            digest.update(self.password.unwrap().as_bytes());
            digest.update(salt.as_bytes());

            digest.finalize()
        };

        let hashed_password = data.as_slice()
            .bytes()
            .flat_map(|x| format!("{:x}", x.unwrap()))
            .collect::<String>();

        self.hashed_password = Some(&hashed_password);
        self.password = None;
    }
}

pub fn get_users<'c>(cfg: &'c Config) -> Vec<User<'c>> {
    let users_files = fs::read_dir("storage/users")?
        .filter_map(|maybe_file| {
            if let Ok(file) = maybe_file {
                if !file.file_type().unwrap().is_file() {
                    log::debug!("skipped not a real file: {:?}", file.path());
                    return None;
                }

                if let Err(why) = file.file_name().to_str() {
                    log::warn!("error while transmuting file name to string: {}", why);
                    return None;
                }

                let file_name = file.file_name().to_str().unwrap();

                if !file_name.starts_with('.') && file_name.ends_with(".toml") {
                    return Some(file);
                }
            }

            log::warn!("users dir entry invalid, skipping");

            None
        });

    users_files.flat_map(|file| {
        let data = fs::read_to_string(file.path());

        if let Err(why) = data {
            log::warn!("couldn't read user file data ({:?}): {}", file.path(), why);
            // return None;
        }

        let data = {
            let data = data.unwrap();
            toml::from_str::<User>(data.as_str())
        };

        if let Err(why) = data {
            log::warn!("invalid user file schema ({:?}): {}", file.path(), why);
            // return None;
        }

        let mut data = data.unwrap();

        data.hash_password(cfg.secret_key);

        {
            let serialized_data = toml::to_string(&data).unwrap();
            log::info!("writing hashed password to {:?}", file.path());
            fs::write(file.path(), serialized_data);
        }

        data
    }).collect()
}