use std::marker::PhantomData;

use axum::extract::{FromRequestParts, Query};
use axum::http::{self, Request};
use axum::response::{IntoResponse, Redirect, Response};
use futures_util::future::BoxFuture;
use openidconnect::{
    AccessToken, AccessTokenHash, AuthorizationCode, GenderClaim, IdToken, IdTokenClaims,
    IdTokenVerifier, JsonWebKey, JweContentEncryptionAlgorithm, JwsSigningAlgorithm,
    OAuth2TokenResponse, PkceCodeVerifier, TokenResponse,
};
use tower_layer::Layer;
use tower_service::Service;
use tower_sessions::Session;

use crate::middleware::oidc::extractor::OidcRpInitiatedLogout;
use crate::middleware::oidc::oidc::ClearSessionFlag;

use super::{
    error::{self, MiddlewareError},
    extractor::{OidcAccessToken, OidcClaims},
    oidc::{AdditionalClaims, AuthenticatedSession, OidcClient, OidcQuery, OidcSession},
    SESSION_KEY,
};

/// Layer for the [`OidcLoginMiddleware`].
#[derive(Clone, Default)]
pub struct OidcLoginLayer<AC>
where
    AC: AdditionalClaims,
{
    additional: PhantomData<AC>,
    post_login_redirect: Option<http::Uri>,
}

impl<AC: AdditionalClaims> OidcLoginLayer<AC> {
    pub fn new() -> Self {
        Self {
            additional: PhantomData,
            post_login_redirect: None,
        }
    }

    pub fn with_post_login_redirect(mut self, uri: Option<http::Uri>) -> Self {
        self.post_login_redirect = uri;
        self
    }
}

impl<I, AC> Layer<I> for OidcLoginLayer<AC>
where
    AC: AdditionalClaims,
{
    type Service = OidcLoginMiddleware<I, AC>;

    fn layer(&self, inner: I) -> Self::Service {
        OidcLoginMiddleware {
            inner,
            additional: PhantomData,
        }
    }
}

/// This middleware forces the user to be authenticated and redirects the user to the OpenID Connect
/// Issuer to authenticate. This Middleware needs to be loaded afer [`OidcAuthMiddleware`].
#[derive(Clone)]
pub struct OidcLoginMiddleware<I, AC>
where
    AC: AdditionalClaims,
{
    inner: I,
    additional: PhantomData<AC>,
}

impl<I, AC, B> Service<Request<B>> for OidcLoginMiddleware<I, AC>
where
    I: Service<Request<B>, Response = Response> + Send + 'static + Clone,
    I::Error: Send + Into<error::BoxError>,
    I::Future: Send + 'static,
    AC: AdditionalClaims,
    B: Send + 'static,
{
    type Response = I::Response;
    type Error = error::MiddlewareError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner
            .poll_ready(cx)
            .map_err(|e| error::MiddlewareError::NextMiddleware(e.into()))
    }

    fn call(&mut self, request: Request<B>) -> Self::Future {
        let inner = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, inner);

        if request.extensions().get::<OidcAccessToken>().is_some() {
            // the OidcAuthMiddleware had a valid id token
            Box::pin(async move {
                let response: Response = inner
                    .call(request)
                    .await
                    .map_err(|e| MiddlewareError::NextMiddleware(e.into()))?;
                Ok(response)
            })
        } else {
            // no valid id token or refresh token was found and the user has to login
            Box::pin(async move {
                let (mut parts, _) = request.into_parts();

                let mut oidc_client: OidcClient<AC> = parts
                    .extensions
                    .get()
                    .cloned()
                    .ok_or(MiddlewareError::AuthMiddlewareNotFound)?;

                let query = Query::<OidcQuery>::from_request_parts(&mut parts, &())
                    .await
                    .ok();

                let session = parts
                    .extensions
                    .get::<Session>()
                    .ok_or(MiddlewareError::SessionNotFound)?;

                let login_session: Option<OidcSession<AC>> = session
                    .get(SESSION_KEY)
                    .await
                    .map_err(MiddlewareError::from)?;

                let handler_uri =
                    strip_oidc_from_path(oidc_client.application_base_url.clone(), &parts.uri)?;

                oidc_client.client = oidc_client
                    .client
                    .set_redirect_uri(openidconnect::RedirectUrl::new(handler_uri.to_string())?);

                if let (Some(mut login_session), Some(query)) = (login_session, query) {
                    // the request has the request headers of the oidc redirect
                    // parse the headers and exchange the code for a valid token

                    if login_session.csrf_token.secret() != &query.state {
                        return Err(MiddlewareError::CsrfTokenInvalid);
                    }

                    let async_client = AsyncHttpClient(oidc_client.http_client.clone());
                    let token_response = oidc_client
                        .client
                        .exchange_code(AuthorizationCode::new(query.code.to_string()))
                        .map_err(error::MiddlewareError::ConfigurationError)?
                        // Set the PKCE code verifier.
                        .set_pkce_verifier(PkceCodeVerifier::new(
                            login_session.pkce_verifier.secret().clone(),
                        ))
                        .request_async(&async_client)
                        .await?;

                    // // Extract the ID token claims after verifying its authenticity and nonce.
                    let id_token = token_response
                        .id_token()
                        .ok_or(MiddlewareError::IdTokenMissing)?;

                    let claims = id_token.claims(
                        &oidc_client.client.id_token_verifier(),
                        &login_session.nonce,
                    )?;

                    validate_access_token_hash(
                        id_token,
                        &oidc_client.client.id_token_verifier(),
                        token_response.access_token(),
                        claims,
                    )?;

                    login_session.authenticated = Some(AuthenticatedSession {
                        id_token: id_token.clone(),
                        access_token: token_response.access_token().clone(),
                    });
                    let refresh_token = token_response.refresh_token().cloned();
                    if let Some(refresh_token) = refresh_token {
                        login_session.refresh_token = Some(refresh_token);
                    }

                    session.insert(SESSION_KEY, login_session).await?;

                    Ok(Redirect::temporary(&handler_uri.to_string()).into_response())
                } else {
                    // generate a login url and redirect the user to it

                    let (pkce_challenge, pkce_verifier) =
                        openidconnect::PkceCodeChallenge::new_random_sha256();
                    let (auth_url, csrf_token, nonce) = {
                        let mut auth = oidc_client.client.authorize_url(
                            openidconnect::core::CoreAuthenticationFlow::AuthorizationCode,
                            openidconnect::CsrfToken::new_random,
                            openidconnect::Nonce::new_random,
                        );

                        for scope in oidc_client.scopes.iter() {
                            auth = auth.add_scope(openidconnect::Scope::new(scope.to_string()));
                        }

                        auth.set_pkce_challenge(pkce_challenge).url()
                    };

                    let oidc_session = OidcSession::<AC> {
                        nonce: nonce,
                        csrf_token: csrf_token,
                        pkce_verifier: pkce_verifier,
                        authenticated: None,
                        refresh_token: None,
                    };

                    session.insert(SESSION_KEY, oidc_session).await?;

                    Ok(Redirect::temporary(auth_url.as_str()).into_response())
                }
            })
        }
    }
}

/// Helper function to remove the OpenID Connect authentication response query attributes from a
/// [`Uri`].
fn strip_oidc_from_path(
    base_url: http::Uri,
    uri: &http::Uri,
) -> Result<http::Uri, error::MiddlewareError> {
    let mut base_url = base_url.into_parts();

    base_url.path_and_query = uri
        .path_and_query()
        .map(|path_and_query| {
            let query = path_and_query
                .query()
                .and_then(|uri| {
                    uri.split('&')
                        .filter(|x| {
                            !x.starts_with("code")
                                && !x.starts_with("state")
                                && !x.starts_with("session_state")
                                && !x.starts_with("iss")
                        })
                        .map(|x| x.to_string())
                        .reduce(|acc, x| acc + "&" + &x)
                })
                .map(|x| format!("?{x}"))
                .unwrap_or_default();

            http::uri::PathAndQuery::from_maybe_shared(format!(
                "{}{}",
                path_and_query.path(),
                query
            ))
        })
        .transpose()?;

    Ok(http::Uri::from_parts(base_url)?)
}

/// Verify the access token hash to ensure that the access token hasn't been substituted for
/// another user's.
/// Returns `Ok` when access token is valid
fn validate_access_token_hash<
    AC: AdditionalClaims,
    GC: GenderClaim,
    JE: JweContentEncryptionAlgorithm<KeyType = JS::KeyType>,
    JS: JwsSigningAlgorithm,
    K: JsonWebKey<SigningAlgorithm = JS>,
>(
    id_token: &IdToken<AC, GC, JE, JS>,
    id_token_verifier: &IdTokenVerifier<K>,
    access_token: &AccessToken,
    claims: &IdTokenClaims<AC, GC>,
) -> Result<(), error::MiddlewareError> {
    if let Some(expected_access_token_hash) = claims.access_token_hash() {
        let key = id_token
            .signing_key(&id_token_verifier)
            .map_err(MiddlewareError::SignatureVerificationError)?;
        let signing_alg = id_token
            .signing_alg()
            .map_err(MiddlewareError::SignatureVerificationError)?;
        let actual_access_token_hash = AccessTokenHash::from_token(access_token, signing_alg, key)?;
        if actual_access_token_hash == *expected_access_token_hash {
            Ok(())
        } else {
            Err(error::MiddlewareError::AccessTokenHashInvalid)
        }
    } else {
        Ok(())
    }
}

/// Layer for the [`OidcAuthMiddleware`]
#[derive(Clone)]
pub struct OidcAuthLayer<AC>
where
    AC: AdditionalClaims,
{
    client: OidcClient<AC>,
}

impl<AC: AdditionalClaims> OidcAuthLayer<AC> {
    pub fn new(client: OidcClient<AC>) -> Self {
        Self { client }
    }
}

impl<I, AC> Layer<I> for OidcAuthLayer<AC>
where
    AC: AdditionalClaims,
{
    type Service = OidcAuthMiddleware<I, AC>;

    fn layer(&self, inner: I) -> Self::Service {
        OidcAuthMiddleware {
            inner,
            client: self.client.clone(),
        }
    }
}

/// This middleware checks if the cached session is valid and injects the Claims, the AccessToken
/// and the OidcClient in the request. This middleware needs to be loaded for every handler that is
/// using one of the Extractors. This middleware **doesn't force a user to be
/// authenticated**.
#[derive(Clone)]
pub struct OidcAuthMiddleware<I, AC>
where
    AC: AdditionalClaims,
{
    inner: I,
    client: OidcClient<AC>,
}

impl<I, AC, B> Service<Request<B>> for OidcAuthMiddleware<I, AC>
where
    I: Service<Request<B>> + Send + 'static + Clone,
    I::Response: IntoResponse + Send,
    I::Error: Send + Into<error::BoxError>,
    I::Future: Send + 'static,
    AC: AdditionalClaims,
    B: Send + 'static,
{
    type Response = Response;
    type Error = MiddlewareError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner
            .poll_ready(cx)
            .map_err(|e| MiddlewareError::NextMiddleware(e.into()))
    }

    fn call(&mut self, request: Request<B>) -> Self::Future {
        let inner = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, inner);
        let oidc_client = self.client.clone();

        Box::pin(async move {
            let (mut parts, body) = request.into_parts();

            let session = parts
                .extensions
                .get::<Session>()
                .ok_or(MiddlewareError::SessionNotFound)?
                .clone();

            let mut login_session: Option<OidcSession<AC>> = session
                .get(SESSION_KEY)
                .await
                .map_err(MiddlewareError::from)?;

            if let Some(login_session) = login_session.as_mut() {
                let id_token_claims = login_session.authenticated.as_ref().and_then(|session| {
                    session
                        .id_token
                        .claims(
                            &oidc_client.client.id_token_verifier(),
                            &login_session.nonce,
                        )
                        .ok()
                        .cloned()
                        .map(|claims| (session, claims))
                });

                if let Some((session, claims)) = id_token_claims {
                    insert_extensions(&mut parts, claims, &oidc_client, &session);
                } else if let Some(refresh_token) = login_session.refresh_token.as_ref() {
                    if let Some((claims, authenticated_session, refresh_token)) =
                        try_refresh_token(&oidc_client, refresh_token, &login_session.nonce).await?
                    {
                        insert_extensions(&mut parts, claims, &oidc_client, &authenticated_session);
                        login_session.authenticated = Some(authenticated_session);

                        if let Some(refresh_token) = refresh_token {
                            login_session.refresh_token = Some(refresh_token);
                        }
                    }

                    // save refreshed session or delete it when the token couldn't be refreshed
                    let session = parts
                        .extensions
                        .get::<Session>()
                        .ok_or(MiddlewareError::SessionNotFound)?;

                    session.insert(SESSION_KEY, login_session).await?
                }
            }

            parts.extensions.insert(oidc_client);

            let request = Request::from_parts(parts, body);
            let response: Response = inner
                .call(request)
                .await
                .map_err(|e| MiddlewareError::NextMiddleware(e.into()))?
                .into_response();

            let has_logout_ext = response.extensions().get::<ClearSessionFlag>().is_some();
            if let (true, Some(mut login_session)) = (has_logout_ext, login_session) {
                login_session.authenticated = None;
                session.insert(SESSION_KEY, login_session).await?;
            }

            Ok(response)
        })
    }
}

fn insert_extensions<AC: AdditionalClaims>(
    parts: &mut http::request::Parts,
    claims: IdTokenClaims<AC, openidconnect::core::CoreGenderClaim>,
    client: &OidcClient<AC>,
    authenticated_session: &AuthenticatedSession<AC>,
) {
    parts.extensions.insert(OidcClaims(claims));
    parts.extensions.insert(OidcAccessToken(
        authenticated_session.access_token.secret().to_string(),
    ));

    let rp_initiated_logout = client
        .end_session_endpoint
        .as_ref()
        .map(|end_session_endpoint| OidcRpInitiatedLogout {
            end_session_endpoint: end_session_endpoint.clone(),
            id_token_hint: authenticated_session.id_token.to_string(),
            client_id: client.client_id.clone(),
            post_logout_redirect_uri: None,
            state: None,
        });

    parts.extensions.insert(rp_initiated_logout);
}

async fn try_refresh_token<AC: AdditionalClaims>(
    oidc_client: &OidcClient<AC>,
    refresh_token: &openidconnect::RefreshToken,
    nonce: &openidconnect::Nonce,
) -> Result<
    Option<(
        IdTokenClaims<AC, openidconnect::core::CoreGenderClaim>,
        AuthenticatedSession<AC>,
        Option<openidconnect::RefreshToken>,
    )>,
    MiddlewareError,
> {
    let mut refresh_request = oidc_client.client.exchange_refresh_token(refresh_token)?;

    for scope in oidc_client.scopes.iter() {
        refresh_request = refresh_request.add_scope(openidconnect::Scope::new(scope.to_string()));
    }

    let async_client = AsyncHttpClient(oidc_client.http_client.clone());
    match refresh_request.request_async(&async_client).await {
        Ok(token_response) => {
            // Extract the ID token claims after verifying its authenticity and nonce.
            let id_token = token_response
                .id_token()
                .ok_or(MiddlewareError::IdTokenMissing)?;
            let claims = id_token.claims(&oidc_client.client.id_token_verifier(), nonce)?;

            validate_access_token_hash(
                id_token,
                &oidc_client.client.id_token_verifier(),
                token_response.access_token(),
                claims,
            )?;

            let authenticated_session = AuthenticatedSession {
                id_token: id_token.clone(),
                access_token: token_response.access_token().clone(),
            };

            Ok(Some((
                claims.clone(),
                authenticated_session,
                token_response.refresh_token().cloned(),
            )))
        }
        Err(openidconnect::RequestTokenError::ServerResponse(e))
            if *e.error() == openidconnect::core::CoreErrorResponseType::InvalidGrant =>
        {
            // Refresh failed, refresh_token most likely expired or
            // invalid, the session can be considered lost
            Ok(None)
        }
        Err(err) => Err(err.into()),
    }
}

/// `openidconnect::reqwest::async_http_client` that uses a custom `reqwest::client`
pub struct AsyncHttpClient(openidconnect::reqwest::Client);

impl<'a> openidconnect::AsyncHttpClient<'a> for AsyncHttpClient {
    type Error = openidconnect::reqwest::Error;
    type Future = BoxFuture<'a, Result<openidconnect::HttpResponse, Self::Error>>;

    fn call(&'a self, request: openidconnect::HttpRequest) -> Self::Future {
        Box::pin(async move {
            let client = &self.0;
            let url = request.uri().to_string();

            let mut request_builder = client
                .request(request.method().into(), url)
                .body(request.body().to_owned());

            for (name, value) in request.headers() {
                request_builder = request_builder.header(name.as_str(), value.as_bytes());
            }

            let request = request_builder
                .build()
                .map_err(openidconnect::reqwest::Error::from)?;

            let response = client
                .execute(request)
                .await
                .map_err(openidconnect::reqwest::Error::from)?;

            let status_code = response.status();
            let headers = response.headers().to_owned();
            let chunks = response
                .bytes()
                .await
                .map_err(openidconnect::reqwest::Error::from)?;

            let mut response = openidconnect::HttpResponse::new(chunks.to_vec());
            *response.status_mut() = status_code;
            *response.headers_mut() = headers;

            Ok(response)
        })
    }
}
