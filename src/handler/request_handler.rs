use crate::dto::request_dto::{RequestReadDto, RequestRegisterDto};
use crate::state::request_state::RequestState;
use axum::{Json, extract::{Extension, Path, State}};
use crate::response::api_response::ApiSuccessResponse;
use crate::entity::request::Request;
use crate::error::{api_error::ApiError, api_request_error::ValidatedRequest};
use crate::error::db_error::DbError;

pub async fn get(
    Extension(request): Extension<Request>,
) -> Json<ApiSuccessResponse<RequestReadDto>> {
    Json(ApiSuccessResponse::send(RequestReadDto::from(request)))
}

pub async fn query(
    State(state): State<RequestState>,
    Path(id): Path<i32>
) -> Result<Json<RequestReadDto>, ApiError> {
    let request: Result<Request, DbError> = state.request_repo.find(id).await;
    return match request {
        Ok(request) => Ok(Json(RequestReadDto::from(request))),
        Err(e) => Err(Json(e.into_response()))
    };
}

pub async fn register(
    State(state): State<RequestState>,
    ValidatedRequest(payload): ValidatedRequest<RequestRegisterDto>,
) -> Result<Json<RequestReadDto>, ApiError> {
    let request = state.request_service.create_request(payload).await?;
    Ok(Json(request))
}
