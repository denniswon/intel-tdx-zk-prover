use crate::error::{
    db_error::DbError,
    attestation_error::AttestationError,
    agent_error::AgentError,
    request_error::RequestError
};
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error(transparent)]
    DbError(#[from] DbError),
    #[error(transparent)]
    RequestError(#[from] RequestError),
    #[error(transparent)]
    AgentError(#[from] AgentError),
    #[error(transparent)]
    AttestationError(#[from] AttestationError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::DbError(error) => error.into_response(),
            ApiError::RequestError(error) => error.into_response(),
            ApiError::AgentError(error) => error.into_response(),
            ApiError::AttestationError(error) => error.into_response(),
        }
    }
}
