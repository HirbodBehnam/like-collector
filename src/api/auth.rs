use lazy_static::lazy_static;
use parking_lot::RwLock;
use uuid::Uuid;
use warp::Reply;

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};

use super::{
    errors::errors::*,
    types::{LoginBody, TokenBody},
};
use crate::database::db::{Database, LoginResultError, ID};

pub struct AddedTime<T> {
    pub data: T,
    added_time: u64,
}

impl<T> From<T> for AddedTime<T> {
    fn from(t: T) -> Self {
        Self {
            data: t,
            added_time: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() + TOKEN_TTL.as_secs(),
        }
    }
}

lazy_static! {
    pub(crate) static ref AUTH_MAP: RwLock<HashMap<String, AddedTime<ID>>> =
        RwLock::new(HashMap::new());
}

/// How much each token must live before being deleted from [`AUTH_MAP`]
const TOKEN_TTL: Duration = Duration::from_secs(60 * 60);

/// This macro tries to authorize user
/// It always returns a [`ID`] which is the user id of user
/// If the token is invalid an unauthorized error is returned as the result of function which this macro is called from
#[macro_export]
macro_rules! authorize_user {
    ($auth:expr) => {
        match AUTH_MAP.read().get($auth) {
            None => return Ok(unauthorized_error()),
            Some(user_id) => user_id.data,
        }
    };
}

/// This macro tries to authorize the user from [`Option<String>`]
/// If the Option is [`None`], it will return [`None`]
/// Otherwise either [`Some(ID)`] is returned where the value in it is the user id of user
/// Or an unauthorized error error is returned as the result of function which is macro runs in
#[macro_export]
macro_rules! authorize_user_optional {
    ($auth:expr) => {
        match $auth {
            None => None,
            Some(auth_string) => match AUTH_MAP.read().get(&auth_string) {
                None => return Ok(unauthorized_error()),
                Some(user_id) => Some(user_id.data),
            },
        }
    };
}

/// This function tries to login a user and sends a token back to it
pub(crate) async fn login_user(
    request: LoginBody,
    db: Arc<Database>,
) -> Result<warp::reply::Response, warp::Rejection> {
    let login_result = db
        .check_user_pass(&request.username, request.password)
        .await;
    match login_result {
        Ok(user_id) => {
            // Create token
            let token = Uuid::new_v4();
            // Save on server
            AUTH_MAP.write().insert(token.to_string(), user_id.into());
            // Return to user
            return Ok(warp::reply::json(&TokenBody { token: &token }).into_response());
        }
        Err(err) => match err {
            LoginResultError::SqlError(_) => {
                return Ok(internal_error());
            }
            LoginResultError::InvalidUsernamePassword => {
                return Ok(error_message("invalid username or password", 400));
            }
        },
    }
}

/// This function will renew user's old token
/// The old token is deleted and a new one is created and returned to user
pub(crate) async fn renew_token(auth: String) -> Result<warp::reply::Response, warp::Rejection> {
    let mut map = AUTH_MAP.write();
    let user_id = match map.remove(&auth) {
        None => return Ok(unauthorized_error()),
        Some(user_id) => user_id,
    };
    // Create token
    let token = Uuid::new_v4();
    // Insert into map
    map.insert(token.to_string(), user_id);
    // Return to user
    return Ok(warp::reply::json(&TokenBody { token: &token }).into_response());
}

/// This function will start a cleanup on AUTH_MAP
/// Each token lives for [`TOKEN_TTL`]
pub(crate) fn start_token_cleanup() {
    tokio::spawn(async {
        loop {
            tokio::time::sleep(TOKEN_TTL).await;
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            AUTH_MAP.write().retain(|_, value| {
                // https://stackoverflow.com/a/45724688/4213397
                value.added_time < now
            });
        }
    });
}
