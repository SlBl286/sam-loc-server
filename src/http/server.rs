use axum::{Router, routing::post};
use sqlx::Error;

use crate::http::auth_handler::{login, register};

pub async fn start_http_server(host: String, db: sqlx::PgPool) ->  Result<(), Error>  {
    let app = Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .with_state(db);

     let listener = tokio::net::TcpListener::bind(host.clone())
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}