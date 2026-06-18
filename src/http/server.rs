use axum::{
    Router, 
    routing::post, 
    middleware::from_fn,
    http::{HeaderValue, Method, Request, header, StatusCode},
    response::Response,
    middleware::Next,
    body::Body,
};
use sqlx::Error;

use crate::http::auth_handler::{login, register};

pub async fn start_http_server(host: String, db: sqlx::PgPool) -> Result<(), Error> {
    let app = Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .layer(from_fn(cors_middleware))
        .with_state(db);

    let listener = tokio::net::TcpListener::bind(host.clone())
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn cors_middleware(req: Request<Body>, next: Next) -> Response {
    let method = req.method().clone();
    
    if method == Method::OPTIONS {
        let mut response = Response::new(Body::empty());
        *response.status_mut() = StatusCode::OK;
        let headers = response.headers_mut();
        headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
        headers.insert(header::ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"));
        headers.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static("Content-Type, Authorization"));
        return response;
    }

    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    headers.insert(header::ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"));
    headers.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static("Content-Type, Authorization"));
    
    response
}
