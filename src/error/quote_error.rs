use crate::response::api_response::ApiErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuoteError {
    #[error("Quote not found")]
    NotFound,
    #[error("Quote invalid")]
    Invalid,
    #[error("Quote unauthorized")]
    Unauthorized,
}

impl IntoResponse for QuoteError {
    fn into_response(self) -> Response {
        let status_code = match self {
            QuoteError::NotFound => StatusCode::NOT_FOUND,
            QuoteError::Invalid => StatusCode::BAD_REQUEST,
            QuoteError::Unauthorized => StatusCode::UNAUTHORIZED,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
