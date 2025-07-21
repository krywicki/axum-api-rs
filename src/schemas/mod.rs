/// Schemas for input and output on endpoints

#[derive(utoipa::ToSchema, serde::Serialize)]
pub struct Health {
    pub status: &'static str,
}

#[derive(utoipa::ToSchema, serde::Deserialize)]
pub struct OAuth2CodeGrantParams {
    pub code: String,
}

#[derive(utoipa::ToSchema, serde::Deserialize)]
pub struct OAuth2TokensBody {
    pub access_token: String,
}
