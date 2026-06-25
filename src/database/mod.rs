pub mod user_repo;

use sqlx::PgPool;

pub async fn init_db(db_url: String) -> Result<PgPool, Box<dyn std::error::Error>> {
    let pool = PgPool::connect(&db_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
