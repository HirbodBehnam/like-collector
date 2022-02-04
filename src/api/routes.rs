use std::{net::SocketAddr, sync::Arc};

use warp::{Filter, Reply};

use crate::{authorize_user, authorize_user_optional, database::db::Database};

use super::{
    auth::{login_user, renew_token, start_token_cleanup, AUTH_MAP},
    errors::errors::*,
    types::*,
};

const MAX_REQUEST_SIZE: u64 = 1024;
const MAX_THREAD_SIZE: u64 = 1024 * 32;
const AUTH_HEADER: &str = "auth";

/// Runs the server with given database
///
/// # Arguments
///
/// * `listen_address`: Which address we should listen on
/// * `database`: The database which we must use
pub async fn run_server(listen_address: &str, database: Database) {
    start_token_cleanup();
    let database = Arc::new(database);
    let database = warp::any().map(move || database.clone());
    let login_endpoint = warp::post()
        .and(warp::path("login"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(MAX_REQUEST_SIZE))
        .and(warp::body::json::<LoginBody>())
        .and(database.clone())
        .and_then(login_user);
    let renew_token_endpoint = warp::post()
        .and(warp::path("renew"))
        .and(warp::path::end())
        .and(warp::header::header::<String>(AUTH_HEADER))
        .and_then(renew_token);
    let get_board = warp::get()
        .and(warp::path("board"))
        .and(warp::path::end())
        .and(warp::query::<GetBoardQuery>())
        .and(database.clone())
        .and(warp::header::optional::<String>(AUTH_HEADER))
        .and_then(get_boards);
    let like_route = warp::path("like")
        .and(warp::path::end())
        .and(warp::header::header::<String>(AUTH_HEADER))
        .and(database.clone())
        .and(warp::query::<LikeRequest>());
    let like_post_route = warp::put().and(like_route.clone()).and_then(like_post);
    let like_post_delete_route = warp::delete().and(like_route).and_then(like_post_delete);
    let post_thread = warp::post()
        .and(warp::path("post"))
        .and(warp::path::end())
        .and(warp::header::header::<String>(AUTH_HEADER))
        .and(database.clone())
        .and(warp::body::content_length_limit(MAX_THREAD_SIZE))
        .and(warp::body::json::<PostThreadBody>())
        .and_then(post_thread);
    let final_routes = login_endpoint
        .or(get_board)
        .or(renew_token_endpoint)
        .or(like_post_route)
        .or(like_post_delete_route)
        .or(post_thread);
    warp::serve(final_routes)
        .run(
            listen_address
                .parse::<SocketAddr>()
                .expect("invalid listen address"),
        )
        .await;
}

/// This function will last 100 boards and sends them to user
/// If the user has an auth header this function will also try to authorize user
/// If the user is authorized the result sent back will also contain if a thread is liked or not
async fn get_boards(
    request: GetBoardQuery,
    db: Arc<Database>,
    auth: Option<String>,
) -> Result<warp::reply::Response, warp::Rejection> {
    // Authorize user
    let user = authorize_user_optional!(auth);
    // Get boards
    let boards = db.get_board_threads(request.from, user).await;
    match boards {
        Ok(data) => Ok(warp::reply::json(&data).into_response()),
        Err(err) => {
            println!("cannot get boards: {}", err);
            Ok(internal_error())
        }
    }
}

async fn like_post(
    auth: String,
    db: Arc<Database>,
    request: LikeRequest,
) -> Result<warp::reply::Response, warp::Rejection> {
    let user_id = authorize_user!(&auth);
    if let Err(err) = db.like_thread(user_id, request.board_id).await {
        if let sqlx::Error::Database(ref db_err) = err {
            let db_err: &sqlx::mysql::MySqlDatabaseError = db_err.downcast_ref();
            if db_err.number() == 1062 {
                // Duplicate like
                return Ok(error_message("duplicate like", 400));
            }
        }
        println!("cannot like: {}", err);
        return Ok(internal_error());
    }
    Ok(empty_json())
}

async fn like_post_delete(
    auth: String,
    db: Arc<Database>,
    request: LikeRequest,
) -> Result<warp::reply::Response, warp::Rejection> {
    let user_id = authorize_user!(&auth);
    if let Err(err) = db.delete_like(user_id, request.board_id).await {
        println!("cannot delete like: {}", err);
        return Ok(internal_error());
    }
    Ok(empty_json())
}

async fn post_thread(
    auth: String,
    db: Arc<Database>,
    request: PostThreadBody,
) -> Result<warp::reply::Response, warp::Rejection> {
    let user_id = authorize_user!(&auth);
    match db.add_post(user_id, &request.text).await {
        Ok(db_index) => {
            Ok(warp::reply::json(&PostThreadResult { post_id: db_index }).into_response())
        }
        Err(err) => {
            println!("cannot create post: {}", err);
            Ok(internal_error())
        }
    }
}
