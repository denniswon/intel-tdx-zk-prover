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
    #[error("Failed to update quote status on success")]
    UpdateStatusOnSuccess,
    #[error("Failed to update quote status on failure")]
    UpdateStatusOnFailure,
    #[error("Failed to submit proof")]
    SubmitProof,
    #[error("Failed to prove")]
    Prove,
    #[error("Failed to verify proof")]
    VerifyProof,
}

impl IntoResponse for QuoteError {
    fn into_response(self) -> Response {
        let status_code = match self {
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
    }
}
