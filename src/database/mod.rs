pub mod user_repo;

use std::env;

use dotenvy::dotenv;
use sqlx::{Error, PgPool};

pub async fn init_db(db_url: String) -> Result<PgPool, Error> {

    PgPool::connect(&db_url).await
}
