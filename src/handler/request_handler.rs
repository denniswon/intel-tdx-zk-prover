use crate::dto::request_dto::{RequestReadDto, RequestRegisterDto};
use crate::entity::request::Request;
use crate::error::db_error::DbError;
use crate::error::{api_error::ApiError, api_request_error::ValidatedRequest};
use crate::repository::request_repository::RequestRepositoryTrait;
use crate::response::api_response::ApiSuccessResponse;
use crate::state::request_state::RequestState;
use axum::{
    extract::{Extension, Path, State},
    Json,
};

pub async fn get(
    Extension(request): Extension<Request>,
) -> Json<ApiSuccessResponse<RequestReadDto>> {
    Json(ApiSuccessResponse::send(RequestReadDto::from(request)))
}

pub async fn query(
    State(state): State<RequestState>,
    Path(id): Path<i32>,
) -> Result<Json<RequestReadDto>, ApiError> {
    let request: Result<Request, DbError> = state.request_repo.find(id.try_into().unwrap()).await;
    match request {
        Ok(request) => Ok(Json(RequestReadDto::from(request))),
        Err(e) => Err(ApiError::DbError(e)),
    }
}

pub async fn register(
    State(state): State<RequestState>,
    ValidatedRequest(payload): ValidatedRequest<RequestRegisterDto>,
) -> Result<Json<RequestReadDto>, ApiError> {
    let request = state.request_service.create(payload).await?;
    Ok(Json(request))
}

pub async fn delete(
    State(state): State<RequestState>,
    Path(id): Path<i32>,
) -> Result<Json<RequestReadDto>, ApiError> {
    let request = state.request_service.delete(id).await?;
    Ok(Json(request))
}
