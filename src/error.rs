use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;

/// Map any Result<T, E> error into a response that can be returned
/// from an axum handler
pub trait ResultToHttpErrResponse<T, E> {
    fn map_err_resp(self, status: StatusCode) -> Result<T, Response>;
    fn map_detail_err_resp(self, status: StatusCode, detail: impl ToString) -> Result<T, Response>;
}

impl<T, E> ResultToHttpErrResponse<T, E> for Result<T, E> {
    fn map_err_resp(self, status: StatusCode) -> Result<T, Response> {
        match self {
            Ok(val) => Ok(val),
            Err(_) => {
                let detail = status.canonical_reason().unwrap_or("unknown").to_string();
                Err((status, detail).into_response())
            }
        }
    }

    fn map_detail_err_resp(self, status: StatusCode, detail: impl ToString) -> Result<T, Response> {
        match self {
            Ok(val) => Ok(val),
            Err(_) => Err((status, detail.to_string()).into_response()),
        }
    }
}
