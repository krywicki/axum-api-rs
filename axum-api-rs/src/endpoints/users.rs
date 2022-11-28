use axum::{
    extract::{Path, State},
    response::{IntoResponse, Json},
};

use slog::{self, b, o};

use crate::{
    models::{Doc, MongoCollection, MongoFilter, User},
    schemas::fields::UserId,
    AppStateType, HttpResult,
};
use log::info;
use mongodb::bson::Document;
use tracing::{event, instrument, Level};

pub async fn get_user(
    Path(id_or_email): Path<UserId>,
    State(state): AppStateType,
) -> HttpResult<impl IntoResponse> {
    let col = User::collection(&state.db);

    let user: Option<Doc<User>> = col.find_one(id_or_email.mongo_filter()?, None).await?;

    //info!(slog::kv!(&user));
    let logger = log::logger();
    slog::info!(logger.into(), "hi");
    Ok(Json(user))
}

pub async fn get_users() -> HttpResult<impl IntoResponse> {
    Ok("hello")
}
