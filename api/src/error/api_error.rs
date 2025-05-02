use prover::error::{
    db_error::DbError,
    quote_error::QuoteError,
    request_error::RequestError,
};
use crate::response::api_response::ApiErrorResponse;
use axum::response::{IntoResponse, Response};
use hyper::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code, clippy::enum_variant_names)]
pub enum ApiError {
    #[error("Invalid UUID: {0}")]
    InvalidUuid(String),
    #[error(transparent)]
    DbError(#[from] DbError),
    #[error(transparent)]
    RequestError(#[from] RequestError),
    #[error(transparent)]
    QuoteError(#[from] QuoteError),
    #[error("Something went wrong: {0}")]
    InvariantViolationError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::InvalidUuid(error) => {
                let status_code = StatusCode::BAD_REQUEST;
                ApiErrorResponse::send(status_code.as_u16(), Some(error.to_string()))
            }
            ApiError::DbError(ref error) => {
                let status_code = match error {
                    DbError::SomethingWentWrong(_error) => StatusCode::INTERNAL_SERVER_ERROR,
                    DbError::UniqueConstraintViolation(_error) => StatusCode::INTERNAL_SERVER_ERROR,
                };
                ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
            },
            ApiError::RequestError(ref error) => {
                let status_code = match error {
                    RequestError::NotFound => StatusCode::NOT_FOUND,
                    RequestError::Invalid => StatusCode::BAD_REQUEST,
                    RequestError::Unauthorized => StatusCode::UNAUTHORIZED,
                };
                ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
            },
            ApiError::QuoteError(ref error) => {
                let status_code = match error {
                    QuoteError::NotFound => StatusCode::NOT_FOUND,
                    QuoteError::Invalid => StatusCode::BAD_REQUEST,
                    QuoteError::Unauthorized => StatusCode::UNAUTHORIZED,
                    QuoteError::UpdateStatusOnSuccess => StatusCode::INTERNAL_SERVER_ERROR,
                    QuoteError::UpdateStatusOnFailure => StatusCode::INTERNAL_SERVER_ERROR,
                    QuoteError::SubmitProof => StatusCode::INTERNAL_SERVER_ERROR,
                    QuoteError::Prove => StatusCode::INTERNAL_SERVER_ERROR,
                    QuoteError::VerifyProof => StatusCode::INTERNAL_SERVER_ERROR,
                };
                ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
            },
            ApiError::InvariantViolationError(error) => {
                let status_code = StatusCode::INTERNAL_SERVER_ERROR;
                ApiErrorResponse::send(status_code.as_u16(), Some(error.to_string()))
            }
        }
    }
}
