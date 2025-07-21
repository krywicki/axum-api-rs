use axum::{
    http::{self, status::StatusCode},
    response::IntoResponse,
};
use openidconnect::{core::CoreErrorResponseType, HttpClientError, StandardErrorResponse};

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, thiserror::Error)]
pub enum MiddlewareError {
    #[error("access token hash invalid")]
    AccessTokenHashInvalid,

    #[error("csrf token invalid")]
    CsrfTokenInvalid,

    #[error("id token missing")]
    IdTokenMissing,

    #[error("signing: {0:?}")]
    Signing(#[from] openidconnect::SigningError),

    #[error("claims verification: {0:?}")]
    ClaimsVerification(#[from] openidconnect::ClaimsVerificationError),

    #[error("url parsing: {0:?}")]
    UrlParsing(#[from] openidconnect::url::ParseError),

    #[error("uri parsing: {0:?}")]
    UriParsing(#[from] http::uri::InvalidUri),

    #[error("uri parts parsing: {0:?}")]
    UriPartsParsing(#[from] http::uri::InvalidUriParts),

    #[error("request token: {0:?}")]
    RequestToken(
        #[from]
        openidconnect::RequestTokenError<
            openidconnect::reqwest::Error,
            StandardErrorResponse<CoreErrorResponseType>,
        >,
    ),

    #[error("session error: {0:?}")]
    Session(#[from] tower_sessions::session::Error),

    #[error("session not found")]
    SessionNotFound,

    #[error("next middleware")]
    NextMiddleware(#[from] BoxError),

    #[error("auth middleware not found")]
    AuthMiddlewareNotFound,

    #[error("problem configuring request")]
    ConfigurationError(#[from] openidconnect::ConfigurationError),

    #[error("signature verification error: {0:?}")]
    SignatureVerificationError(#[from] openidconnect::SignatureVerificationError),
}

#[derive(Debug, thiserror::Error)]
pub enum ExtractorError {
    #[error("unauthorized")]
    Unauthorized,

    #[error("rp initiated logout not supported by issuer")]
    RpInitiatedLogoutNotSupported,

    #[error("could not build rp initiated logout uri")]
    FailedToCreateRpInitiatedLogoutUri,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("url parsing: {0:?}")]
    UrlParsing(#[from] UrlParseError),

    #[error("invalid end_session_endpoint uri: {0:?}")]
    InvalidEndSessionEndpoint(http::uri::InvalidUri),

    #[error("discovery: {0:?}")]
    Discovery(
        #[from] openidconnect::DiscoveryError<HttpClientError<openidconnect::reqwest::Error>>,
    ),

    #[error("extractor: {0:?}")]
    Extractor(#[from] ExtractorError),

    #[error("extractor: {0:?}")]
    Middleware(#[from] MiddlewareError),
}

#[derive(Debug, thiserror::Error)]
pub enum UrlParseError {
    #[error("url parsing: {0:?}")]
    OidcParse(#[from] openidconnect::url::ParseError),
    #[error("url parsing: {0:?}")]
    AxumParse(#[from] axum::http::uri::InvalidUri),
}

impl IntoResponse for ExtractorError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized").into_response(),
            Self::RpInitiatedLogoutNotSupported => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
            }
            Self::FailedToCreateRpInitiatedLogoutUri => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
            }
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        log::error!("{:#?}", &self);

        match self {
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response(),
        }
    }
}

impl IntoResponse for MiddlewareError {
    fn into_response(self) -> axum::response::Response {
        log::error!("{:#?}", &self);
        match self {
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response(),
        }
    }
}
