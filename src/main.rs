use std::env;

mod database;
mod api;

#[tokio::main]
async fn main() {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be specified");
    let listen_address = env::var("LISTEN").expect("LISTEN must be specified");
    let db = database::db::Database::new(&database_url).await;
    api::routes::run_server(&listen_address, db).await;
}
