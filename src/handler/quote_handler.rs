use crate::dto::quote_dto::QuoteReadDto;
use crate::{dto::quote_dto::QuoteRegisterDto, error::api_request_error::ValidatedRequest};
use crate::entity::quote::TdxQuote;
use crate::entity::dcap::DcapVerifiedOutput;
use crate::error::db_error::DbError;
use crate::error::api_error::ApiError;
use crate::repository::quote_repository::QuoteRepositoryTrait;
use crate::response::api_response::ApiSuccessResponse;
use crate::sp1::prove::DcapProof;
use crate::state::quote_state::QuoteState;
use axum::{
    extract::{Extension, Path, State},
    Json,
};
use sqlx::types::Uuid;

pub async fn get(
    Extension(quote): Extension<TdxQuote>,
) -> Json<ApiSuccessResponse<QuoteReadDto>> {
    Json(ApiSuccessResponse::send(QuoteReadDto::from(quote)))
}

pub async fn query(
    State(state): State<QuoteState>,
    Path(id): Path<String>,
) -> Result<Json<QuoteReadDto>, ApiError> {
    match Uuid::parse_str(&id) {
        Ok(id) => {
            let quote: Result<TdxQuote, DbError> =
                state.quote_repo.find(id).await;
            match quote {
                Ok(quote) => Ok(Json(QuoteReadDto::from(quote))),
                Err(e) => Err(ApiError::DbError(e)),
            }
        }
        Err(e) => Err(ApiError::InvalidUuid(e.to_string())),
    }
}

pub async fn register(
    State(state): State<QuoteState>,
    ValidatedRequest(payload): ValidatedRequest<QuoteRegisterDto>,
) -> Result<Json<QuoteReadDto>, ApiError> {
    let quote = state
        .quote_service
        .create_quote(payload)
        .await?;
    Ok(Json(QuoteReadDto::from(quote)))
}

pub async fn verify_dcap(
    State(state): State<QuoteState>,
    Path(id): Path<String>,
) -> Result<Json<DcapVerifiedOutput>, ApiError> {
    match Uuid::parse_str(&id) {
        Ok(id) => {
            let quote: Result<TdxQuote, DbError> =
                state.quote_repo.find(id).await;
            match quote {
                Ok(quote) => {
                    let tcb = state.quote_service.verify_dcap(quote, None);
                    match tcb {
                        Ok(tcb) => Ok(Json(DcapVerifiedOutput::from_output(tcb))),
                        Err(e) => Err(ApiError::QuoteError(e)),
                    }
                }
                Err(e) => Err(ApiError::DbError(e)),
            }
        }
        Err(e) => Err(ApiError::InvalidUuid(e.to_string())),
    }
}

pub async fn prove(
    State(state): State<QuoteState>,
    Path(id): Path<String>,
) -> Result<Json<DcapProof>, ApiError> {
    match Uuid::parse_str(&id) {
        Ok(id) => {
            let proof = state.quote_service.prove(id).await;
            match proof {
                Ok(proof) => Ok(Json(proof)),
                Err(e) => Err(ApiError::QuoteError(e)),
            }
        }
        Err(e) => Err(ApiError::InvalidUuid(e.to_string())),
    }
}

pub async fn verify(
    State(state): State<QuoteState>,
    ValidatedRequest(payload): ValidatedRequest<DcapProof>,
) -> Result<Json<DcapVerifiedOutput>, ApiError> {
    let output = state.quote_service.verify(payload).await;
    match output {
        Ok(output) => Ok(Json(DcapVerifiedOutput::from_output(output))),
        Err(e) => Err(ApiError::QuoteError(e)),
    }
}

pub async fn submit_proof(
    State(state): State<QuoteState>,
    ValidatedRequest(payload): ValidatedRequest<DcapProof>,
) -> Result<Json<DcapVerifiedOutput>, ApiError> {
    let output = state.quote_service.submit_proof(payload).await;
    match output {
        Ok(output) => Ok(Json(DcapVerifiedOutput::from_output(output))),
        Err(e) => Err(ApiError::QuoteError(e)),
    }
}
