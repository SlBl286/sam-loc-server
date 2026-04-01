use std::{env, io::Error as IoError};

use dotenvy::dotenv;

use crate::{database::init_db, http::server::start_http_server, lobby::start_ws_server};

#[tokio::main]
async fn main() -> Result<(), IoError> {
    dotenv().ok();
    let host = env::var("HOST").expect("HOST must be set");
    let ws_host = env::var("WS_HOST").expect("WS_HOST must be set");

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = init_db(db_url)
        .await
        .expect("Failed to connect to the database");

    let http = tokio::spawn(start_http_server(host, db));
    let ws = tokio::spawn(start_ws_server(ws_host));

    let _ = tokio::join!(http, ws);
    Ok(())
}

mod auth;
mod database;
mod http;
mod lobby;
mod network;
