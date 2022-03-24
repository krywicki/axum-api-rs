use axum::{response::Json, routing::get, Router};
use serde_json::{json, Value};

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route("/hello-world", get(get_hello_world));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_hello_world() -> Json<Value> {
    Json(json!({"message": "hello world!"}))
}
