use axum::{
    extract::{Extension, Path, Query},
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};

mod endpoints;
mod error;
mod models;
mod schemas;

use mongodb::Client;
use schemas::GetUser;

use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_client = Client::with_uri_str("mongodb://admin:admin@127.0.0.1:27017")
        .await
        .expect("can't connect to database")
        .database("production");

    // build our application with a single route
    let app = Router::new()
        .route("/hello-world", get(get_hello_world))
        .route("/users/:id_or_email", get(get_user))
        .route("/users", get(get_users))
        .layer(Extension(db_client));

    // 404 fallback service
    let app = app.fallback(handler_404.into_service());

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn get_user(Path(id_or_email): Path<String>) -> impl IntoResponse {
    let user = schemas::GetUser {
        first_name: "Joe".into(),
        last_name: "Krywicki".into(),
        email: "joe.krywicki@mail.com".into(),
        id: "0001".into(),
    };

    let id_or_email = Value::from(id_or_email);
    if id_or_email != user.id && id_or_email != user.email {
        return (
            StatusCode::NOT_FOUND,
            Json(
                json!({"message": "User not found", "detail": format!("User {} could not be found", id_or_email)}),
            ),
        );
    }

    (StatusCode::OK, Json(user))
}

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
