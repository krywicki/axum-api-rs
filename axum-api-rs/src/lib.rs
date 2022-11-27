use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use mongodb::Database;
use std::sync::Arc;

pub type AppStateType = State<Arc<AppState>>;

pub struct AppState {
    pub db: Database,
}

pub mod endpoints;
pub mod error;
pub mod models;
pub mod schemas;

pub type HttpResult<T> = Result<T, error::HttpError>;
