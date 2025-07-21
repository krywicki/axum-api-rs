use crate::{
    error::ResultToHttpErrResponse,
    schemas,
    state::{AppState, AppStateOpenApiRouter, ArcAppState},
    utils::LogResult,
};
use axum::{
    extract::{Query, Request, State},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Json,
};

use diesel_async::RunQueryDsl;
use reqwest::StatusCode;
use serde_json::{json, Map, Value as JsonValue};
use serde_valid::json::ToJsonString;
use utoipa_axum::{router::OpenApiRouter, routes};

/// Routes that are confined to the base path '/'

pub fn get_routes() -> AppStateOpenApiRouter {
    OpenApiRouter::new().routes(routes!(health, logout))
}

/// Server health status check
#[utoipa::path(get, path="/health", responses((status=OK, body=schemas::Health)))]
pub async fn health() -> Json<schemas::Health> {
    Json(schemas::Health { status: "OK" })
}

/// Redirect to login page

// #[utoipa::path(get, path = "/login")]
// pub async fn login(State(state): State<ArcAppState>) -> Redirect {
//     let mut url = state.openid_configuration().authorization_endpoint.clone();
//     let oauth2 = &state.config().oauth2;

//     url.query_pairs_mut().extend_pairs([
//         ("client_id", oauth2.client_id.as_str()),
//         ("redirect_url", oauth2.redirect_url.as_str()),
//         ("response_type", "code"),
//         ("audience", oauth2.audience.as_str()),
//     ]);

//     Redirect::temporary(url.as_str())
// }

#[utoipa::path(get, path = "/logout")]
pub async fn logout() -> impl IntoResponse {
    (StatusCode::OK, ())
}

// #[utoipa::path(post, path = "/oauth-callback", request_body=())]
// pub async fn oauth_callback(
//     State(state): State<ArcAppState>,
//     params: Query<schemas::OAuth2CodeGrantParams>,
// ) -> Result<Redirect, Response> {
//     let token_endpoint = &state.openid_configuration().token_endpoint;
//     let oauth2 = &state.config().oauth2;

//     let client = reqwest::Client::new();
//     let resp = client
//         .post(token_endpoint.to_string())
//         .json(&json!({
//             "grant_type": "authorization_code",
//             "client_id": oauth2.client_id.as_str(),
//             "client_secret": oauth2.client_secret.as_str(),
//             "redirect_uri": oauth2.redirect_url.as_str(),
//             "code": params.code.as_str()
//         }))
//         .send()
//         .await
//         .log_err_msg("Failed sending access/authorization code exchange request")
//         .map_err_resp(StatusCode::INTERNAL_SERVER_ERROR)?;

//     let tokens: schemas::OAuth2TokensBody = resp
//         .error_for_status()
//         .log_err_msg("Failed access/authorization code exchange request")
//         .map_err_resp(StatusCode::INTERNAL_SERVER_ERROR)?
//         .json()
//         .await
//         .log_err_msg("Failed parsing oauth-callback tokens")
//         .map_err_resp(StatusCode::INTERNAL_SERVER_ERROR)?;

//     log::info!("oauth-callback access_token({})", tokens.access_token);

//     Ok(Redirect::to("/docs"))
// }
