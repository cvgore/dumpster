use std::fmt::Write;
use std::ops::Add;
use std::sync::Arc;
use std::time::{Duration, Instant};

use rocket::State;
use rocket::form::Form;
use rocket::http::Status;
use rocket::serde::json::{json, Value};
use rocket_governor::{Method, Quota, RocketGovernable, RocketGovernor};

use crate::AppState;
use crate::files::UserToken;

#[derive(FromForm, Debug)]
pub struct LoginData<'r> {
    user: &'r str,
    pass: &'r str,
}

pub type Token = Arc<str>;

fn new_token() -> String {
    use argon2::password_hash::rand_core::RngCore;

    const TOKEN_BYTES: usize = 32;

    let token_bytes = {
        let mut buf = [0u8; TOKEN_BYTES];

        rand::thread_rng().fill_bytes(&mut buf);

        buf
    };

    let token = {
        let mut s = String::with_capacity(TOKEN_BYTES);

        for byte in token_bytes {
            write!(&mut s, "{:x}", byte).expect("stringification of token failed");
        }

        s
    };

    token
}

pub struct LoginRateLimitGuard;

impl<'r> RocketGovernable<'r> for LoginRateLimitGuard {
    fn quota(_method: Method, _route_name: &str) -> Quota {
        const WAIT_TIME: u64 = 2 * 60;
        const MAX_BURST: u32 = 5;

        Quota::with_period(Duration::from_secs(WAIT_TIME))
            .unwrap()
            .allow_burst(Self::nonzero(MAX_BURST))
    }
}

#[post("/login", data = "<form>")]
pub async fn login(form: Form<LoginData<'_>>, state: &State<AppState>, _rt: RocketGovernor<'_, LoginRateLimitGuard>) -> Result<Value, Status> {
    let entry = state.users.get(form.user);

    if entry.is_none() {
        log::debug!("user '{}' not found", form.user);

        return Err(Status::Unauthorized);
    }

    let user = entry.unwrap().clone();

    if !user.check_password(form.pass) {
        log::debug!("password for user '{}' mismatch", form.user);

        return Err(Status::Unauthorized);
    }

    let mut tokens = state.tokens.write().await;

    let token = new_token();

    tokens.list.insert(token.clone().into_boxed_str().into(), user);
    tokens.lifespans.insert(
        token.clone().into_boxed_str().into(),
        Instant::now().add(Duration::from_secs(5 * 60)),
    );

    Ok(json!({
        "token": token
    }))
}

#[post("/logout")]
pub async fn logout(ut: UserToken, state: &State<AppState>) -> Status {
    let mut tokens = state.tokens.write().await;

    tokens.list.remove(&ut.token);
    tokens.lifespans.remove(&ut.token);

    Status::Ok
}