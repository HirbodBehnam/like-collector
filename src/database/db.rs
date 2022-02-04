use std::time::Duration;
use tokio::task;

use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};

use super::types::BoardData;
pub struct Database {
    pool: Pool<MySql>,
}

pub enum LoginResultError {
    SqlError(sqlx::Error),
    InvalidUsernamePassword,
}

impl From<sqlx::Error> for LoginResultError {
    fn from(error: sqlx::Error) -> Self {
        LoginResultError::SqlError(error)
    }
}

/// ID represents the ID type stored in database
pub type ID = u32;

impl Database {
    /// This function creates a new database connection pool and returns the pool.
    ///
    /// # Panics
    ///
    /// Panics if it can't connect to database.
    pub async fn new(uri: &str) -> Self {
        let pool = MySqlPoolOptions::new()
            .max_connections(150)
            .connect_timeout(Duration::from_secs(2))
            .connect(uri)
            .await
            .expect("cannot connect to database");
        Self { pool }
    }

    /// This function gets the username and password of a user and checks if its username and password are correct
    /// If the username or password is wrong returns [`LoginResultError::InvalidUsernamePassword`]
    /// If everything is good this returns the ID of the user
    pub async fn check_user_pass(
        &self,
        username: &str,
        password: String,
    ) -> Result<ID, LoginResultError> {
        // Check if it exists
        let hashed_password: Option<(ID, String)> =
            sqlx::query_as("SELECT `id`, `password` FROM `users` WHERE `username`=?")
                .bind(username)
                .fetch_optional(&self.pool)
                .await?;
        // Check password
        match hashed_password {
            Some(hashed_password) => {
                // Spawn on another thread because this is async context
                let result = task::spawn_blocking(move || {
                    match bcrypt::verify(password, &hashed_password.1) {
                        Ok(b) => return b,
                        Err(err) => {
                            println!("error on bcyrpt verify: {}", err);
                            false
                        }
                    }
                })
                .await;
                match result {
                    Ok(ok) => match ok {
                        true => return Ok(hashed_password.0),
                        false => return Err(LoginResultError::InvalidUsernamePassword),
                    },
                    Err(_) => return Err(LoginResultError::InvalidUsernamePassword),
                }
            }
            None => return Err(LoginResultError::InvalidUsernamePassword),
        }
    }

    /// Gets the last 100 threads of board
    ///
    /// # Arguments
    /// * `from`: The thread ID to get the boards before it. Zero means get from last one.
    /// * `user_id`: If this request comes from a logged in user we can send back if they have liked the thread or not
    ///
    /// # Returns
    /// All boards
    pub async fn get_board_threads(
        &self,
        mut from: ID,
        user_id: Option<ID>,
    ) -> Result<Vec<BoardData>, sqlx::Error> {
        // Fix from if needed
        if from == 0 {
            from = ID::MAX;
        }
        // Get boards
        match user_id {
            None => {
                sqlx::query_as::<_, BoardData>("SELECT `id`, `data`, (SELECT COUNT(*) FROM `likes` WHERE `board_id`=`board`.`id`) as `likes` FROM `board` WHERE `id` <= ? ORDER BY `id` DESC LIMIT 100")
                .bind(from)
                .fetch_all(&self.pool).await
            }
            Some(user_id) => {
                sqlx::query_as::<_, BoardData>("SELECT `id`, `data`, (SELECT COUNT(*) FROM `likes` WHERE `board_id`=`board`.`id`) as `likes`, (SELECT EXISTS (SELECT * FROM `likes` WHERE `liker`=? AND `board_id`=`board`.`id`)) as `liked` FROM `board` WHERE `id` <= ? ORDER BY `id` DESC LIMIT 100")
                .bind(user_id)
                .bind(from)
                .fetch_all(&self.pool).await
            }
        }
    }

    /// This function will like a thread for user
    pub async fn like_thread(&self, user_id: ID, thread_id: ID) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO `likes` (`board_id`,`liker`) VALUES (?,?)")
            .bind(thread_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// This function will delete a like for user. Note that if the like does not exists this function will do nothing
    /// And won't return an error
    pub async fn delete_like(&self, user_id: ID, thread_id: ID) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM `likes` WHERE `board_id`=? AND `liker`=?")
            .bind(thread_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// This function will add a post to board and returns it's ID
    pub async fn add_post(&self, user_id: ID, post_data: &str) -> Result<ID, sqlx::Error> {
        let inserted = sqlx::query("INSERT INTO `board` (`creator`,`data`) VALUES (?,?)")
            .bind(user_id)
            .bind(post_data)
            .execute(&self.pool)
            .await?;
        Ok(inserted.last_insert_id() as u32)
    }
}
