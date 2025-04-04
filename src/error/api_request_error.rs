use crate::response::api_response::ApiErrorResponse;
use async_trait::async_trait;
use axum::body::Bytes;
use axum::{body::HttpBody,http::Request, BoxError, Json};
use axum::extract::{rejection::JsonRejection, FromRequest};
use axum::response::{IntoResponse, Response};
use serde::de::DeserializeOwned;
use thiserror::Error;
use validator::Validate;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),
    #[error(transparent)]
    JsonRejection(#[from] JsonRejection),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedRequest<T>(pub T);

#[async_trait]
impl<T, S, B> FromRequest<S, B> for ValidatedRequest<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = ServerError;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedRequest(value))
    }
}

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedRequest<T>
where
    Bytes: FromRequest<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Form(value) = Form::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedForm(value))
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            ServerError::ValidationError(_) => {
                ApiErrorResponse::send(400, Some(self.to_string().replace('\n', ", ")))
            }
            ServerError::JsonRejection(_) => ApiErrorResponse::send(400, Some(self.to_string())),
        }
    }
}
