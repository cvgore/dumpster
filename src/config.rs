use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config<'c> {
    pub(crate) secret_key: &'c str,
}