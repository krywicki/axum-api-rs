use std::sync::Arc;

use diesel_async::pooled_connection::{
    bb8::{Pool, PooledConnection},
    AsyncDieselConnectionManager,
};

use utoipa_axum::router::OpenApiRouter;

use crate::{
    config::{AppConfig, OidcConfig},
    oidc,
    utils::{LogResult, OpenidConfiguration, OpenidConfigurationErr},
};
use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata},
    AuthType,
};

pub type AsyncSqliteConnection<'a> =
    PooledConnection<'a, SyncConnectionWrapper<diesel::SqliteConnection>>;
pub type AsyncSqliteConnectionManager =
    AsyncDieselConnectionManager<SyncConnectionWrapper<diesel::SqliteConnection>>;
pub type AsyncSqlitePool = Pool<SyncConnectionWrapper<diesel::SqliteConnection>>;

pub type ArcAppState = Arc<AppState>;

pub type AppStateOpenApiRouter = OpenApiRouter<crate::state::ArcAppState>;

pub struct AppState {
    config: AppConfig,
    sqlite_pool: AsyncSqlitePool,
}

impl AppState {
    pub async fn from_config(config: AppConfig) -> Result<Self, anyhow::Error> {
        let sqlite_db_file = config.sqlite_db_file.clone();

        let manager = AsyncSqliteConnectionManager::new(sqlite_db_file.to_string_lossy());
        let db_pool = AsyncSqlitePool::builder()
            .build(manager)
            .await
            .map_err(|err| AppStateError::SqlitePoolError(err.to_string()))
            .log_err()?;

        Ok(Self {
            config: config.clone(),
            sqlite_pool: db_pool,
        })
    }

    pub async fn sqlite_connection(&self) -> Result<AsyncSqliteConnection<'_>, AppStateError> {
        Ok(self
            .sqlite_pool
            .get()
            .await
            .map_err(|err| AppStateError::SqlitePoolError(err.to_string()))
            .log_err()?)
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AppStateError {
    #[error("Error with sqlite connection pool - {0}")]
    SqlitePoolError(String),
    #[error("{0}")]
    OpenidConfiguartionError(#[from] OpenidConfigurationErr),
}

#[derive(thiserror::Error, Debug)]
pub enum AppConfigError {
    #[error("Err deserializing config vars")]
    DeserializeErr(#[from] config::ConfigError),

    #[error("Validation err")]
    ValidationErr(String),

    #[error("Unknown error")]
    Unknown,
}
