use crate::response::api_response::ApiErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AttestationError {
    #[error("Attestation not found")]
    AttestationNotFound,
    #[error("Attestation invalid")]
    AttestationInvalid,
    #[error("Attestation unauthorized")]
    AttestationUnauthorized,
}

impl IntoResponse for AttestationError {
    fn into_response(self) -> Response {
        let status_code = match self {
            AttestationError::AttestationNotFound => StatusCode::NOT_FOUND,
            AttestationError::AttestationInvalid => StatusCode::BAD_REQUEST,
            AttestationError::AttestationUnauthorized => StatusCode::UNAUTHORIZED,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
