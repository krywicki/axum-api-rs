/// Openid Connect layer
/// *note: Heavily influenced by [axum_oidc](https://github.com/pfzetto/axum-oidc/tree/master) crate
use std::str::FromStr;

use axum::http::Uri;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use openidconnect::{
    core::{
        CoreAuthDisplay, CoreAuthPrompt, CoreClaimName, CoreClaimType, CoreClientAuthMethod,
        CoreErrorResponseType, CoreGenderClaim, CoreGrantType, CoreJsonWebKey,
        CoreJweContentEncryptionAlgorithm, CoreJweKeyManagementAlgorithm, CoreJwsSigningAlgorithm,
        CoreResponseMode, CoreResponseType, CoreRevocableToken, CoreSubjectIdentifierType,
        CoreTokenIntrospectionResponse, CoreTokenType,
    },
    AccessToken, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields, EndpointMaybeSet,
    EndpointNotSet, EndpointSet, IdTokenFields, IssuerUrl, Nonce, PkceCodeVerifier, RefreshToken,
    StandardErrorResponse, StandardTokenResponse,
};

use super::error;

type OidcTokenResponse<AC> = StandardTokenResponse<
    IdTokenFields<
        AC,
        EmptyExtraTokenFields,
        CoreGenderClaim,
        CoreJweContentEncryptionAlgorithm,
        CoreJwsSigningAlgorithm,
    >,
    CoreTokenType,
>;

type IdToken<AZ> = openidconnect::IdToken<
    AZ,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJwsSigningAlgorithm,
>;

pub type Client<AC> = openidconnect::Client<
    AC,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJsonWebKey,
    CoreAuthPrompt,
    StandardErrorResponse<CoreErrorResponseType>,
    OidcTokenResponse<AC>,
    CoreTokenIntrospectionResponse,
    CoreRevocableToken,
    openidconnect::core::CoreRevocationErrorResponse,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

type ProviderMetadata = openidconnect::ProviderMetadata<
    AdditionalProviderMetadata,
    CoreAuthDisplay,
    CoreClientAuthMethod,
    CoreClaimName,
    CoreClaimType,
    CoreGrantType,
    CoreJweContentEncryptionAlgorithm,
    CoreJweKeyManagementAlgorithm,
    CoreJsonWebKey,
    CoreResponseMode,
    CoreResponseType,
    CoreSubjectIdentifierType,
>;

pub trait AdditionalClaims:
    openidconnect::AdditionalClaims + Clone + Sync + Send + Serialize + DeserializeOwned
{
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdditionalProviderMetadata {
    end_session_endpoint: Option<String>,
}

impl openidconnect::AdditionalProviderMetadata for AdditionalProviderMetadata {}

#[derive(Clone, Copy)]
pub struct ClearSessionFlag;

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "AC: Serialize + DeserializeOwned")]
pub struct OidcSession<AC: AdditionalClaims> {
    pub nonce: Nonce,
    pub csrf_token: CsrfToken,
    pub pkce_verifier: PkceCodeVerifier,
    pub authenticated: Option<AuthenticatedSession<AC>>,
    pub refresh_token: Option<RefreshToken>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "AC: Serialize + DeserializeOwned")]
pub struct AuthenticatedSession<AC: AdditionalClaims> {
    pub id_token: IdToken<AC>,
    pub access_token: AccessToken,
}

/// response data of the openid issuer after login
#[derive(Debug, Deserialize)]
pub struct OidcQuery {
    pub code: String,
    pub state: String,
    #[allow(dead_code)]
    pub session_state: Option<String>,
}

/// an empty struct to be used as the default type for the additional claims generic
#[derive(Deserialize, Serialize, Debug, Clone, Copy, Default)]
pub struct EmptyAdditionalClaims {}
impl AdditionalClaims for EmptyAdditionalClaims {}
impl openidconnect::AdditionalClaims for EmptyAdditionalClaims {}

#[derive(Clone)]
pub struct OidcClient<AC: AdditionalClaims> {
    pub scopes: Vec<String>,
    pub client_id: String,
    pub client: Client<AC>,
    pub http_client: openidconnect::reqwest::Client,
    pub application_base_url: Uri,
    pub end_session_endpoint: Option<Uri>,
}

impl<AC: AdditionalClaims> OidcClient<AC> {
    pub async fn from_config(config: &crate::config::AppConfig) -> Result<Self, error::Error> {
        let issuer_url = IssuerUrl::new(config.oidc.issuer_url.clone().into())
            .map_err(error::UrlParseError::OidcParse)?;

        let http_client = openidconnect::reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("Failed to build reqwest client");

        let provider_metadata = ProviderMetadata::discover_async(issuer_url, &http_client)
            .await
            .map_err(error::Error::Discovery)?;

        Self::from_provider_metadata(
            provider_metadata,
            Uri::from_str(config.app_base_url.as_str()).map_err(error::UrlParseError::AxumParse)?,
            config.oidc.client_id.clone(),
            config.oidc.client_secret.clone(),
            config.oidc.scopes.clone(),
        )
    }

    pub fn from_provider_metadata(
        provider_metadata: ProviderMetadata,
        application_base_url: Uri,
        client_id: String,
        client_secret: Option<String>,
        scopes: Vec<String>,
    ) -> Result<Self, error::Error> {
        let end_session_endpoint = provider_metadata
            .additional_metadata()
            .end_session_endpoint
            .clone()
            .map(Uri::from_maybe_shared)
            .transpose()
            .map_err(error::Error::InvalidEndSessionEndpoint)?;

        let client = Client::from_provider_metadata(
            provider_metadata,
            ClientId::new(client_id.clone()),
            client_secret.map(ClientSecret::new),
        );

        Ok(Self {
            scopes: scopes,
            client_id: client_id,
            client: client,
            http_client: reqwest::Client::default(),
            application_base_url: application_base_url,
            end_session_endpoint: end_session_endpoint,
        })
    }
}
