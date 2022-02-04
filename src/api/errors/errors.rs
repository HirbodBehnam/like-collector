use serde::Serialize;
use warp::Reply;

/// Simple error message container for json formatting
#[derive(Serialize)]
struct ErrorMessage<'a> {
    error: &'a str,
}

/// This function will return a response with 500 error code and internal error text
/// Must be used when there is a bug in backend
#[inline]
pub fn internal_error() -> warp::reply::Response {
    error_message("internal error", 500)
}

/// This function will create a response for users which are not authorized
#[inline]
pub fn unauthorized_error() -> warp::reply::Response {
    error_message("unauthorized", 401)
}

/// This function will create a response with given message as error and custom status code
#[inline]
pub fn error_message(message: &str, status: u16) -> warp::reply::Response {
    warp::http::Response::builder()
        .status(status)
        .header(warp::http::header::CONTENT_TYPE, "application/json")
        .body(serde_json::to_string(&ErrorMessage { error: message }).unwrap())
        .into_response()
}

/// This function creates a response which contains an empty json and 200 as status code
#[inline]
pub fn empty_json() -> warp::reply::Response {
    warp::http::Response::builder()
        .header(warp::http::header::CONTENT_TYPE, "application/json")
        .body("{}")
        .into_response()
}
