use std::{env, ops::Deref, path};

use serde::Deserialize;
use serde_valid::Validate;

#[derive(Debug, Clone)]
pub struct Uri(axum::http::Uri);

impl Default for Uri {
    fn default() -> Self {
        Self(axum::http::Uri::default())
    }
}

impl Deref for Uri {
    type Target = axum::http::Uri;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> serde::Deserialize<'de> for Uri {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let uri = String::deserialize(deserializer)?;
        let uri: axum::http::Uri = uri.parse().map_err(|err| {
            serde::de::Error::custom(format!("Failed parsing URI({uri}) - {err}"))
        })?;

        Ok(Uri(uri))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct OAuth2Config {
    #[serde(default = "String::new")]
    pub client_id: String,
    #[serde(default = "String::new")]
    pub client_secret: String,
    #[serde(default = "Uri::default")]
    pub redirect_url: Uri,
    #[serde(default = "String::new")]
    pub scopes: String,
    #[serde(default = "Uri::default")]
    pub openid_configuration_url: Uri,
    #[serde(default = "String::new")]
    pub audience: String,
}

#[derive(Validate, Debug, Deserialize, Clone)]
pub struct AppConfig {
    #[validate(min_length = 1)]
    #[validate(max_length = 255)]
    #[serde(default = "AppConfig::_default_host")]
    pub host: String,
    #[validate(minimum = 80)]
    #[validate(maximum = 65_535)]
    #[serde(default = "AppConfig::_default_port")]
    pub port: u64,
    pub oauth2: OAuth2Config,
    #[serde(default = "AppConfig::_default_sqlite_db_file")]
    pub sqlite_db_file: path::PathBuf,
}

impl AppConfig {
    pub fn get_bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    fn _default_host() -> String {
        "localhost".into()
    }

    fn _default_port() -> u64 {
        8000
    }

    fn _default_sqlite_db_file() -> path::PathBuf {
        path::PathBuf::from("./api.db3")
    }
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

impl AppConfig {
    pub fn init() -> Result<Self, AppConfigError> {
        let env_file = env::var("API_DOTENV_FILE").unwrap_or(".env".into());

        dotenvy::from_filename(env_file).ok();

        let c = config::Config::builder()
            .add_source(config::Environment::with_prefix("api"))
            .build()
            .unwrap_or_else(|err| panic!("Missing/Incorrect environment variables - {err}"));

        let cfg: AppConfig = c
            .try_deserialize()
            .map_err(AppConfigError::DeserializeErr)?;

        Ok(cfg)
    }
}
