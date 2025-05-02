use prover::dto::request_dto::RequestReadDto;
use prover::entity::request::OnchainRequest;
use prover::error::db_error::DbError;
use prover::repository::request_repository::OnchainRequestRepositoryTrait;
use prover::state::request_state::RequestState;
use axum::{
    extract::{Extension, Path, State},
    Json,
};
use uuid::Uuid;
use crate::error::api_error::ApiError;
use crate::response::api_response::ApiSuccessResponse;

pub async fn get(
    Extension(request): Extension<OnchainRequest>,
) -> Json<ApiSuccessResponse<RequestReadDto>> {
    Json(ApiSuccessResponse::send(RequestReadDto::from(request)))
}

pub async fn query(
    State(state): State<RequestState>,
    Path(id): Path<String>,
) -> Result<Json<RequestReadDto>, ApiError> {
    match Uuid::parse_str(&id) {
        Ok(id) => {
            let request: Result<OnchainRequest, DbError> = state.request_repo.find(id).await;
            match request {
                Ok(request) => Ok(Json(RequestReadDto::from(request))),
                Err(e) => Err(ApiError::DbError(e)),
            }
        }
        Err(e) => Err(ApiError::InvalidUuid(e.to_string())),
    }
}
