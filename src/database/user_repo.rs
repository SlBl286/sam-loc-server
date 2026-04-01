use sqlx::PgPool;

pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
}

pub async fn get_user_by_username(
    db: &PgPool,
    username: &str,
) -> Option<User> {
    sqlx::query_as!(
        User,
        "SELECT id, username, password_hash FROM users WHERE username = $1",
        username
    )
    .fetch_optional(db)
    .await
    .ok()
    .flatten()
}

pub async fn create_user(
    db: &PgPool,
    username: &str,
    password_hash: &str,
) {
    sqlx::query!(
        "INSERT INTO users (username, password_hash) VALUES ($1, $2)",
        username,
        password_hash
    )
    .execute(db)
    .await
    .unwrap();
}