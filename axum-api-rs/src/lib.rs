use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use mongodb::Database;
use serde::Serialize;
use std::sync::Arc;

pub type AppStateType = State<Arc<AppState>>;

#[derive(Debug)]
pub struct AppState {
    pub db: Database,
}

pub mod endpoints;
pub mod error;
pub mod models;
pub mod schemas;

pub type HttpResult<T> = Result<T, error::HttpError>;

pub fn as_json<T: Serialize>(val: &T) -> serde_json::Value {
    serde_json::to_value(val).unwrap()
}
