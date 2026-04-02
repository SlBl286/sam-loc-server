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

    Ok(Json(LoginResponse {
        token,
        user_id: user.id,
        username: user.username,
        res_type: "login".into(),
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
