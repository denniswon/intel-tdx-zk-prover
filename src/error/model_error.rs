use crate::response::api_response::ApiErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Model not found")]
    ModelNotFound,
    #[error("Model invalid")]
    ModelInvalid,
    #[error("Request unauthorized")]
    RequestUnauthorized,
}

impl IntoResponse for ModelError {
    fn into_response(self) -> Response {
        let status_code = match self {
            ModelError::ModelNotFound => StatusCode::NOT_FOUND,
            ModelError::ModelInvalid => StatusCode::BAD_REQUEST,
            ModelError::RequestUnauthorized => StatusCode::UNAUTHORIZED,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
