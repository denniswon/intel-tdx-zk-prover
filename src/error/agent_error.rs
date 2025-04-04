use crate::response::api_response::ApiErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Agent not found")]
    AgentNotFound,
    #[error("Agent invalid")]
    AgentInvalid,
    #[error("Agent unauthorized")]
    AgentUnauthorized,
}

impl IntoResponse for AgentError {
    fn into_response(self) -> Response {
        let status_code = match self {
            AgentError::AgentNotFound => StatusCode::NOT_FOUND,
            AgentError::AgentInvalid => StatusCode::BAD_REQUEST,
            AgentError::AgentUnauthorized => StatusCode::UNAUTHORIZED,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
