use crate::{
    error::{
        agent_error::AgentError, attestation_error::AttestationError, db_error::DbError,
        request_error::RequestError,
    },
    response::api_response::ApiErrorResponse,
};
use alloy::signers::k256::elliptic_curve::Error;
use axum::response::{IntoResponse, Response};
use hyper::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code, clippy::enum_variant_names)]
pub enum ApiError {
    #[error(transparent)]
    DbError(#[from] DbError),
    #[error(transparent)]
    RequestError(#[from] RequestError),
    #[error(transparent)]
    AgentError(#[from] AgentError),
    #[error(transparent)]
    AttestationError(#[from] AttestationError),
    #[error(transparent)]
    UnknownVariantError(#[from] Error),
    #[error("Something went wrong: {0}")]
    InvariantViolationError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::DbError(error) => error.into_response(),
            ApiError::RequestError(error) => error.into_response(),
            ApiError::AgentError(error) => error.into_response(),
            ApiError::AttestationError(error) => error.into_response(),
            ApiError::UnknownVariantError(error) => {
                let status_code = StatusCode::INTERNAL_SERVER_ERROR;
                ApiErrorResponse::send(status_code.as_u16(), Some(error.to_string()))
            }
            ApiError::InvariantViolationError(error) => {
                let status_code = StatusCode::INTERNAL_SERVER_ERROR;
                ApiErrorResponse::send(status_code.as_u16(), Some(error.to_string()))
            }
        }
    }
}
