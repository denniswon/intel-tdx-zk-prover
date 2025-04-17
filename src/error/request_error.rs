use crate::response::api_response::ApiErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("Request not found")]
    NotFound,
    #[error("Request invalid")]
    Invalid,
    #[error("Request unauthorized")]
    Unauthorized,
}

impl IntoResponse for RequestError {
    fn into_response(self) -> Response {
        let status_code = match self {
            RequestError::NotFound => StatusCode::NOT_FOUND,
            RequestError::Invalid => StatusCode::BAD_REQUEST,
            RequestError::Unauthorized => StatusCode::UNAUTHORIZED,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
