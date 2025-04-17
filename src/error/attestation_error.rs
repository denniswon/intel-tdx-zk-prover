use crate::response::api_response::ApiErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AttestationError {
    #[error("Attestation not found")]
    NotFound,
    #[error("Attestation invalid")]
    Invalid,
    #[error("Attestation unauthorized")]
    Unauthorized,
}

impl IntoResponse for AttestationError {
    fn into_response(self) -> Response {
        let status_code = match self {
            AttestationError::NotFound => StatusCode::NOT_FOUND,
            AttestationError::Invalid => StatusCode::BAD_REQUEST,
            AttestationError::Unauthorized => StatusCode::UNAUTHORIZED,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
