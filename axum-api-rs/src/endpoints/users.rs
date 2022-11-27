use axum::{
    extract::{Path, State},
    response::{IntoResponse, Json},
};
use mongodb::bson::Document;

use crate::{
    models::{MongoCollection, MongoFilter, User},
    schemas::fields::UserId,
    AppStateType, HttpResult,
};

pub async fn get_user(
    Path(id_or_email): Path<UserId>,
    State(state): AppStateType,
) -> HttpResult<impl IntoResponse> {
    let col = User::collection::<Document>(&state.db);

    let user = col.find_one(id_or_email.mongo_filter()?, None).await?;

    Ok(Json(user))
}

pub async fn get_users() -> HttpResult<impl IntoResponse> {
    Ok("hello")
}
