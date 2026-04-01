mod user_repo;


use sqlx::{Database, Error, PgPool};

pub async fn init_db() -> Result<PgPool,Error> {
    PgPool::connect("postgres://postgres:password@localhost/game_server")
        .await
}