use std::{env, io::Error as IoError};

use dotenvy::dotenv;

use crate::lobby::start_server;

#[tokio::main]
async fn main() -> Result<(), IoError> {
    dotenv().ok();
    let host = env::var("HOST").expect("HOST must be set");

    let _ = start_server(host).await;

    Ok(())
}

mod lobby;
mod network;
mod auth;
mod database;