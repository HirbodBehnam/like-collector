use serde::{Deserialize, Serialize};

use crate::database::db::ID;

#[derive(Serialize)]
pub struct TokenBody<'a> {
    pub token: &'a uuid::Uuid,
}

#[derive(Deserialize)]
pub struct LoginBody {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct GetBoardQuery {
    #[serde(default)]
    pub from: ID,
}

#[derive(Deserialize)]
pub struct LikeRequest {
    pub board_id: ID,
}

#[derive(Deserialize)]
pub struct PostThreadBody {
    pub text: String,
}

#[derive(Serialize)]
pub struct PostThreadResult {
    pub post_id: ID,
}
