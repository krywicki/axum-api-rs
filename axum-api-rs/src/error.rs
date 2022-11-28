use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde_json::{json, Value};

use std::{borrow::Cow, error::Error, fmt};

#[derive(Debug)]
pub struct HttpError {
    pub status: StatusCode,
    pub message: String,
    pub detail: Option<serde_json::Value>,
}

impl HttpError {
    pub fn builder() -> HttpErrorBuilder {
        HttpErrorBuilder::default()
    }
}

pub struct HttpErrorBuilder {
    status: StatusCode,
    message: String,
    detail: Option<serde_json::Value>,
}

impl Default for HttpErrorBuilder {
    fn default() -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: "".into(),
            detail: None,
        }
    }
}

impl HttpErrorBuilder {
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    pub fn message<T: Into<String>>(mut self, message: T) -> Self {
        self.message = message.into();
        self
    }

    pub fn detail<T>(mut self, detail: T) -> Self
    where
        T: Into<Option<serde_json::Value>>,
    {
        self.detail = detail.into();
        self
    }

    pub fn build(self) -> HttpError {
        HttpError {
            status: self.status,
            message: self.message,
            detail: self.detail,
        }
    }
}

impl Error for HttpError {}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = json!({
            "status": self.status.as_u16().to_string(),
            "message":self.message,
            "detail":self.detail,
        });

        write!(f, "\n{}", serde_json::to_string_pretty(&json).unwrap())
    }
}

impl From<mongodb::error::Error> for HttpError {
    fn from(err: mongodb::error::Error) -> Self {
        println!("{}", err);
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: StatusCode::INTERNAL_SERVER_ERROR.to_string(),
            detail: None,
        }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let content = json!({
            "message": self.message,
            "detail": self.detail
        });

        //Response::builder().
        (self.status, Json(content)).into_response()
    }
}

// pub enum ErrorCode {
//     InvalidPathPart,
//     InvalidQueryParam,
// }

// impl Into<String> for ErrorCode {
//     fn into(self) -> String {
//         self.to_string()
//     }
// }

// impl fmt::Display for ErrorCode {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         let val = match *self {
//             Self::InvalidPathPart => "INVALID_PATH_PART",
//             Self::ResourceNotFound => "RESOURCE_NOT_FOUND",
//             Self::InvalidQueryParam => "INVALID_QUERY_PARAM",
//             Self::ValidationError => "VALIDATION_ERROR",
//             Self::InvalidBody => "INVALID_BODY",
//             Self::InternalServerError => "INTERNAL_SERVER_ERROR",
//         };

//         write!(f, "{}", val)
//     }
// }
