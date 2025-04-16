use crate::response::api_response::ApiErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum AgentError {
    #[error("Agent not found")]
    NotFound,
    #[error("Agent invalid")]
    Invalid,
    #[error("Agent unauthorized")]
    Unauthorized,
}

impl IntoResponse for AgentError {
    fn into_response(self) -> Response {
        let status_code = match self {
            AgentError::NotFound => StatusCode::NOT_FOUND,
            AgentError::Invalid => StatusCode::BAD_REQUEST,
            AgentError::Unauthorized => StatusCode::UNAUTHORIZED,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
