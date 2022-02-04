use serde::Serialize;

/// BoardData contains the information about a board thread
#[derive(sqlx::FromRow, Serialize)]
pub struct BoardData {
    // The id of the thread
    id: u32,
    // data is the text in the thread
    data: String,
    // How many users have liked this thread
    likes: i64,
    // Is the user linked this or not
    #[sqlx(default)]
    #[serde(skip_serializing_if = "is_false")]
    liked: bool,
}

fn is_false(b: &bool) -> bool {
    !b
}
