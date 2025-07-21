pub mod error;
pub mod extractor;
pub mod middleware;
pub mod oidc;

/// For use in tower-sessions
pub const SESSION_KEY: &str = "axum-oidc";
