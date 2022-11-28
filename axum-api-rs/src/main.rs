use axum::{
    extract::{Extension, Path, Query},
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};

use mongodb::Client;
use std::sync::Arc;

use api::{endpoints as ep, AppState};
use log::info;
use serde_json::{json, Value};
use serdeconv;
use sloggers::{terminal::TerminalLoggerBuilder, Config, LoggerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: LoggerConfig =
        serdeconv::from_toml_file("axum-api-rs/log.toml").map_err(|_| "failed to open log.toml")?;
    let logger = config.build_logger()?;
    let _guard = sloggers::set_stdlog_logger(logger.clone())?;

    let app = Router::new()
        .route("/hello-world", get(get_hello_world))
        .route("/users/:id_or_email", get(ep::users::get_user))
        .route("/users", get(get_users))
        .fallback(handler_404)
        .with_state(Arc::new(AppState {
            db: Client::with_uri_str("mongodb://admin:admin@127.0.0.1:27017")
                .await
                .expect("can't connect to database")
                .database("production"),
        }));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

// async fn get_user(Path(id_or_email): Path<String>) -> impl IntoResponse {
//     let user = schemas::GetUser {
//         first_name: "Joe".into(),
//         last_name: "Krywicki".into(),
//         email: "joe.krywicki@mail.com".into(),
//         id: "0001".into(),
//     };

//     let id_or_email = Value::from(id_or_email);
//     if id_or_email != user.id && id_or_email != user.email {
//         return (
//             StatusCode::NOT_FOUND,
//             Json(
//                 json!({"message": "User not found", "detail": format!("User {} could not be found", id_or_email)}),
//             ),
//         );
//     }

//     (StatusCode::OK, Json(user))
// }

async fn get_users() -> impl IntoResponse {
    (StatusCode::OK, Json(json!([])))
}

async fn get_hello_world() -> impl IntoResponse {}

async fn handler_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({"message": StatusCode::NOT_FOUND.canonical_reason()})),
    )
}
