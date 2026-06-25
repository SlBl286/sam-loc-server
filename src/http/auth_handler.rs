use axum::http::StatusCode;
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth::jwt::*;
use crate::auth::password::*;
use crate::database::user_repo::*;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub res_type: String,
    pub token: String,
    pub user_id: i64,
    pub username: String,
    pub display_name: String,
    pub avatar_url: String,
    pub gold: i64,
}

pub async fn login(
    State(db): State<PgPool>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>,  (StatusCode, String)> {
    let user = get_user_by_username(&db, &req.username)
        .await
        .ok_or((StatusCode::NOT_FOUND,"User not found".to_string()))?;

    if !verify_password(&req.password, &user.password_hash) {
        return Err((StatusCode::UNAUTHORIZED, "Invalid password".to_string()));
    }

    let token = create_token(user.id);

    let profile = check_and_update_weekly_bonus(&db, user.id as u64)
        .await
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Failed to load user profile".to_string()))?;

    Ok(Json(LoginResponse {
        token,
        user_id: user.id,
        username: user.username,
        res_type: "login".into(),
        display_name: profile.display_name,
        avatar_url: profile.avatar_url,
        gold: profile.gold,
    }))
}

pub async fn register(
    State(db): State<PgPool>,
    Json(req): Json<LoginRequest>,
) -> Result<String, String> {
    let hash = hash_password(&req.password);

    create_user(&db, &req.username, &hash).await;

    Ok("ok".into())
}
