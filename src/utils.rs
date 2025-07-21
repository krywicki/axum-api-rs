use serde::Deserialize;

use url::Url;

pub trait LogResult<T, E> {
    /// Logs error message as `'{err}'` format, only on Err results. Returns Result
    fn log_err(self) -> Self;
    /// Logs error message as `'{msg} - {err}'` format, only on Err results. Returns Result
    fn log_err_msg(self, msg: impl AsRef<str>) -> Self;
    /// Logs  ok message as `'{msg}'` format, only on Ok results. Returns Result
    fn log_ok_msg(self, msg: impl AsRef<str>) -> Self;
}

impl<T, E> LogResult<T, E> for Result<T, E>
where
    E: std::fmt::Display,
{
    fn log_ok_msg(self, msg: impl AsRef<str>) -> Self {
        let msg = msg.as_ref();
        match &self {
            Ok(_) => log::info!("{msg}"),
            _ => {}
        }
        self
    }

    fn log_err_msg(self, msg: impl AsRef<str>) -> Self {
        match &self {
            Ok(_) => {}
            Err(err) => {
                let msg = msg.as_ref();
                log::error!("{msg} - {err}");
            }
        }

        self
    }

    fn log_err(self) -> Self {
        match &self {
            Ok(_) => {}
            Err(err) => {
                log::error!("{err}");
            }
        }
        self
    }
}

#[derive(thiserror::Error, Debug)]
pub enum OpenidConfigurationErr {
    #[error("Failed to fetch openid configuration from url({0}) - {1}")]
    FetchErr(String, String),
    #[error("Failed to deserialize openid configuration - {0}")]
    DeserializeErr(#[from] reqwest::Error),
}

#[derive(Debug, Deserialize, Clone)]
pub struct OpenidConfiguration {
    pub issuer: Url,
    pub authorization_endpoint: Url,
    pub token_endpoint: Url,
    pub jwks_uri: Url,
}

impl OpenidConfiguration {
    pub async fn from_url(url: &Url) -> Result<OpenidConfiguration, OpenidConfigurationErr> {
        let resp = reqwest::get(url.as_str()).await;

        Ok(resp
            .map_err(|err| OpenidConfigurationErr::FetchErr(url.to_string(), err.to_string()))
            .log_err()?
            .json::<OpenidConfiguration>()
            .await
            .map_err(|err| OpenidConfigurationErr::DeserializeErr(err))
            .log_err()?)
    }
}
