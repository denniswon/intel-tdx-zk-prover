use crate::dto::request_dto::{UserReadDto, UserRegisterDto};
use crate::error::{api_error::ApiError, request_error::ValidatedRequest};
use crate::state::request_state::UserState;
use axum::{extract::State, Json};

pub async fn register(
    State(state): State<UserState>,
    ValidatedRequest(payload): ValidatedRequest<UserRegisterDto>,
) -> Result<Json<UserReadDto>, ApiError> {
    let user = state.user_service.create_user(payload).await?;
    Ok(Json(user))
}
