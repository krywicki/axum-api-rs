use std::sync::Arc;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
};

mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod routes;
mod schemas;
mod state;
mod utils;

use serde_json::json;
use utoipa::OpenApi;
use utoipa_swagger_ui::{Config as SwaggerConfig, SwaggerUi};

use crate::state::AppStateOpenApiRouter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::AppConfig::init().unwrap();
    let bind_addr = config.get_bind_addr();
    let shared_state = Arc::new(state::AppState::from_config(config).await?);

    env_logger::init();

    let (app, api): (axum::Router, utoipa::openapi::OpenApi) =
        AppStateOpenApiRouter::with_openapi(ApiDoc::openapi())
            .merge(routes::base::get_routes())
            .with_state(shared_state)
            .fallback(handler_404)
            .split_for_parts();

    let swagger_config = SwaggerConfig::new(["/openapi.json"]).query_config_enabled(true);

    let app = app.merge(
        SwaggerUi::new("/docs")
            .config(swagger_config)
            .url("/openapi.json", api),
    );

    let listener = tokio::net::TcpListener::bind(bind_addr.as_str()).await?;

    log::info!("Serving api at http://{bind_addr} (docs @ http://{bind_addr}/docs) ...");
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

async fn handler_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({"detail": StatusCode::NOT_FOUND.canonical_reason()})),
    )
}

#[derive(utoipa::OpenApi)]
#[openapi(info(title = "API", description = "An axum based api for learning."))]
struct ApiDoc;
