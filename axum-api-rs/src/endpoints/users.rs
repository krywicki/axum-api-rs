use axum::{
    extract::{Extension, Path, Query},
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Json},
    Router,
};
use mongodb::Database;

use crate::{
    models::User,
    schemas::{self, fields::UserId},
};

pub fn get_user(
    Path(user_id): Path<UserId>,
    Extension(db): Extension<Database>,
) -> impl IntoResponse {


    db.
    (StatusCode::OK, "OK")
}
