use std::sync::Arc;

use axum::{
    extract::{Extension, Path},
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};

mod config;
mod db;
mod error;
mod handlers;
mod routes;
mod schemas;
mod state;
mod utils;

use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::AppConfig::init().unwrap();
    let bind_addr = config.get_bind_addr();
    let shared_state = Arc::new(state::AppState::from_config(config));

    env_logger::init();

    // build our application with a single route
    let app = Router::new().with_state(shared_state);
    // .route("/hello-world", get(get_hello_world))
    // .route("/users/:id_or_email", get(get_user))
    // .route("/users", get(get_users))

    let app = app.fallback(handler_404);
    let listener = tokio::net::TcpListener::bind(bind_addr.as_str()).await?;

    log::info!("Serving api at http://{bind_addr}...");
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

async fn handler_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({"detail": StatusCode::NOT_FOUND.canonical_reason()})),
    )
}
