use crate::dto::request_dto::{UserReadDto, UserRegisterDto};
use crate::error::{api_error::ApiError, request_error::ValidatedRequest};
use crate::state::model_state::ModelState;
use axum::{Json, extract::State};

pub async fn register(
    State(state): State<ModelState>,
    ValidatedRequest(payload): ValidatedRequest<ModelRegisterDto>,
) -> Result<Json<ModelReadDto>, ApiError> {
    let model = state.model_service.create_model(payload).await?;
    Ok(Json(model))
}
