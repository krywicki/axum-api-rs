use std::{env, ops::Deref, path, str::FromStr};

use axum::http;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Deserializer};
use serde_valid::Validate;

#[derive(Debug, Clone, PartialEq)]
pub struct Uri(http::Uri);

#[derive(thiserror::Error, Debug)]
pub enum UriError {
    #[error("Invalid URI - {0}")]
    InvalidUri(#[from] http::uri::InvalidUri),
    #[error("Invalid URI Parts - {0}")]
    InvalidUriParts(#[from] http::uri::InvalidUriParts),
}

impl AsRef<Uri> for Uri {
    fn as_ref(&self) -> &Uri {
        &self
    }
}

impl Default for Uri {
    fn default() -> Self {
        Self(http::Uri::default())
    }
}

impl Deref for Uri {
    type Target = http::Uri;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&str> for Uri {
    type Error = http::uri::InvalidUri;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self(http::Uri::from_str(value)?))
    }
}

impl TryFrom<String> for Uri {
    type Error = UriError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str()).map_err(UriError::InvalidUri)
    }
}

impl<'de> serde::de::Deserialize<'de> for Uri {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let val = String::deserialize(deserializer)?;

        let uri = http::Uri::from_maybe_shared(val).map_err(serde::de::Error::custom)?;
        Ok(Uri(uri))
    }
}

impl Into<http::Uri> for Uri {
    fn into(self) -> http::Uri {
        self.0
    }
}

impl TryInto<openidconnect::url::Url> for Uri {
    type Error = openidconnect::url::ParseError;

    fn try_into(self) -> Result<openidconnect::url::Url, Self::Error> {
        openidconnect::url::Url::parse(self.0.to_string().as_str())
    }
}

impl Uri {
    pub fn join_uri(&self, uri: impl AsRef<Uri>) -> Result<Uri, UriError> {
        let uri = uri.as_ref();
        let mut base_parts = self.0.clone().into_parts();

        let base_path = self.path().strip_suffix("/").unwrap_or(self.path());
        let new_path = uri.path().strip_prefix("/").unwrap_or(uri.path());

        let joined_path = format!("{base_path}/{new_path}");

        base_parts.path_and_query = Some(joined_path.parse()?);

        Ok(Self(http::Uri::from_parts(base_parts)?))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct OidcConfig {
    #[serde(default = "String::new")]
    pub client_id: String,
    #[serde(default = "default_string_opt_none")]
    pub client_secret: Option<String>,
    #[serde(default = "OidcConfig::_default_scopes")]
    pub scopes: Vec<String>,
    #[serde(default = "String::new")]
    pub audience: String,
    pub issuer_url: Uri,
    #[serde(skip, default = "OidcConfig::_default_oauth_redirect_url")]
    oauth_redirect_url: Uri,
}

impl OidcConfig {
    fn _default_scopes() -> Vec<String> {
        vec![]
    }

    fn _default_oauth_redirect_url() -> Uri {
        Uri::try_from("/oauth-redirect").expect("Invalid Uri")
    }

    pub fn post_deserialize(&mut self, app_base_url: &Uri) -> Result<(), AppConfigError> {
        self.post_deserialize_oauth_redirect_url(app_base_url)?;

        Ok(())
    }

    /// If oauth_redirect_url is just a relative route (e.g. '/oauth-callback', '/callback', etc)
    /// then prefix the api base url to it.
    fn post_deserialize_oauth_redirect_url(
        &mut self,
        app_base_url: &Uri,
    ) -> Result<(), AppConfigError> {
        match self.oauth_redirect_url.scheme() {
            Some(_) => Ok(()),
            None => {
                let oauth_redirect_url = app_base_url
                    .join_uri(&self.oauth_redirect_url)
                    .map_err(PostDeserializeError::Uri)?;

                self.oauth_redirect_url = oauth_redirect_url;
                Ok(())
            }
        }
    }
}

#[derive(Validate, Debug, Deserialize, Clone)]
pub struct AppConfig {
    #[validate(min_length = 1)]
    #[validate(max_length = 255)]
    #[serde(default = "AppConfig::_default_bind_host")]
    pub bind_host: String,
    #[validate(minimum = 80)]
    #[validate(maximum = 65_535)]
    #[serde(default = "AppConfig::_default_bind_port")]
    pub bind_port: u64,
    pub oidc: OidcConfig,
    #[serde(default = "AppConfig::_default_sqlite_db_file")]
    pub sqlite_db_file: path::PathBuf,
    #[serde(default = "Uri::default")]
    pub base_url: Uri,
}

impl AppConfig {
    pub fn get_bind_addr(&self) -> String {
        format!("{}:{}", self.bind_host, self.bind_port)
    }

    fn _default_bind_host() -> String {
        "localhost".into()
    }

    fn _default_bind_port() -> u64 {
        8000
    }

    fn _default_sqlite_db_file() -> path::PathBuf {
        path::PathBuf::from("./api.db3")
    }

    fn post_deserialize(&mut self) -> Result<(), AppConfigError> {
        self.post_deserialize_base_url()?;

        Ok(())
    }

    /// if api base url not explicitly set, default to (http://localhost or http://127.0.0.1)
    /// depending on bind_host and bind_port values
    fn post_deserialize_base_url(&mut self) -> Result<(), AppConfigError> {
        if self.base_url == Uri::default() {
            let authority = match self.bind_host.trim().to_lowercase().as_str() {
                "localhost" => "localhost",
                "127.0.0.1" => "127.0.0.1",
                _ => "localhost",
            };

            let mut uri = format!("http://{authority}");
            if self.bind_port != 80 {
                uri = format!("{uri}:{}", self.bind_port);
            }

            self.base_url = Uri::try_from(uri).map_err(PostDeserializeError::Uri)?;
        }
        Ok(())
    }

    fn post_deserialize_oidc_config(&mut self) -> Result<(), AppConfigError> {
        let app_base_url = &self.base_url;
        self.oidc.post_deserialize(&app_base_url)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AppConfigError {
    #[error("Err deserializing config vars")]
    DeserializeErr(#[from] config::ConfigError),
    #[error("Post deserialize config error - {0}")]
    PostDeserialize(#[from] PostDeserializeError),
}

#[derive(thiserror::Error, Debug)]
pub enum PostDeserializeError {
    #[error("{0}")]
    Uri(#[from] UriError),
}

impl AppConfig {
    pub fn init() -> Result<Self, AppConfigError> {
        let env_file = env::var("API_DOTENV_FILE").unwrap_or(".env".into());
        dotenvy::from_filename(env_file).ok();

        let c = config::Config::builder()
            .add_source(config::Environment::with_prefix("api").separator("_"))
            .build()
            .unwrap_or_else(|err| panic!("Missing/Incorrect environment variables - {err}"));

        let mut cfg: AppConfig = c
            .try_deserialize()
            .map_err(AppConfigError::DeserializeErr)?;

        cfg.post_deserialize()?;

        Ok(cfg)
    }
}

fn default_string_opt_none() -> Option<String> {
    None
}
