use sqlx::PgPool;

pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String, 
}

pub struct UserProfile {
    pub display_name: String,
    pub avatar_url: String,
    pub gold: i64,
}

pub async fn get_user_by_username(db: &PgPool, username: &str) -> Option<User> {
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

pub async fn create_user(db: &PgPool, username: &str, password_hash: &str) {
    sqlx::query!(
        "INSERT INTO users (username, password_hash, display_name) VALUES ($1, $2, $3)",
        username,
        password_hash,
        username
    )
    .execute(db)
    .await
    .unwrap();
}

pub async fn get_user_profile(db: &PgPool, user_id: u64) -> Option<UserProfile> {
    let user = sqlx::query!(
        "SELECT username, display_name, avatar_url, gold FROM users WHERE id = $1",
        user_id as i64
    )
    .fetch_optional(db)
    .await
    .ok()
    .flatten()?;

    Some(UserProfile {
        display_name: user.display_name.unwrap_or(user.username),
        avatar_url: user.avatar_url.unwrap_or_default(),
        gold: user.gold,
    })
}

pub async fn check_and_update_weekly_bonus(db: &PgPool, user_id: u64) -> Option<UserProfile> {
    let _ = sqlx::query!(
        "UPDATE users
         SET gold = gold + FLOOR(EXTRACT(EPOCH FROM (now() - last_weekly_bonus)) / 604800.0)::BIGINT * 100000,
             last_weekly_bonus = last_weekly_bonus + FLOOR(EXTRACT(EPOCH FROM (now() - last_weekly_bonus)) / 604800.0)::BIGINT * INTERVAL '1 week'
         WHERE id = $1 AND now() >= last_weekly_bonus + INTERVAL '1 week'",
        user_id as i64
    )
    .execute(db)
    .await;

    get_user_profile(db, user_id).await
}
