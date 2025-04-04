use crate::error::{db_error::DbError, agent_error::ModelError, request_error::RequestError };
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error(transparent)]
    DbError(#[from] DbError),
    #[error(transparent)]
    RequestError(#[from] RequestError),
    #[error(transparent)]
    ModelError(#[from] ModelError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::DbError(error) => error.into_response(),
            ApiError::RequestError(error) => error.into_response(),
            ApiError::ModelError(error) => error.into_response(),
        }
    }
}
