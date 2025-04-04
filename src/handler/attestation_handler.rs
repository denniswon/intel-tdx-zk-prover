use crate::dto::attestation_dto::{AttestationReadDto, AttestationRegisterDto};
use crate::state::attestation_state::AttestationState;
use axum::{Json, extract::{Extension, Path, State}};
use crate::response::api_response::ApiSuccessResponse;
use crate::entity::attestation::Attestation;
use crate::error::{api_error::ApiError, api_request_error::ValidatedRequest};
use crate::error::db_error::DbError;
use crate::repository::attestation_repository::AttestationRepositoryTrait;

pub async fn get(
    Extension(attestation): Extension<Attestation>,
) -> Json<ApiSuccessResponse<AttestationReadDto>> {
    Json(ApiSuccessResponse::send(AttestationReadDto::from(attestation)))
}

pub async fn query(
    State(state): State<AttestationState>,
    Path(id): Path<i32>
) -> Result<Json<AttestationReadDto>, ApiError> {
    let attestation: Result<Attestation, DbError> = state.attestation_repo.find(id).await;
    return match attestation {
        Ok(attestation) => Ok(Json(AttestationReadDto::from(attestation))),
        Err(e) => Err(ApiError::DbError(e))
    };
}

pub async fn register(
    State(state): State<AttestationState>,
    ValidatedRequest(payload): ValidatedRequest<AttestationRegisterDto>,
) -> Result<Json<AttestationReadDto>, ApiError> {
    let attestation = state.attestation_service.create_attestation(payload).await?;
    Ok(Json(attestation))
}

