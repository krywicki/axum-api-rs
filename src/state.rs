use r2d2_sqlite::SqliteConnectionManager;

use crate::config::AppConfig;

pub type SqliteConnection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

pub struct AppState {
    config: AppConfig,
    sqlite_pool: r2d2::Pool<SqliteConnectionManager>,
}

impl AppState {
    pub fn from_config(config: AppConfig) -> Self {
        let sqlite_db_file = config.sqlite_db_file.clone();
        let db_manager = SqliteConnectionManager::file(sqlite_db_file);
        let db_pool = r2d2::Pool::new(db_manager).expect("Failed to create sqlite connection pool");

        Self {
            config: config,
            sqlite_pool: db_pool,
        }
    }

    pub fn sqlite_connection(&self) -> SqliteConnection {
        self.sqlite_pool
            .get()
            .expect("Failed to get pooled SQLite connection")
    }
}
