use crate::dto::agent_dto::{AgentReadDto, AgentRegisterDto};
use crate::entity::agent::Agent;
use crate::error::db_error::DbError;
use crate::error::{api_error::ApiError, api_request_error::ValidatedRequest};
use crate::repository::agent_repository::AgentRepositoryTrait;
use crate::response::api_response::ApiSuccessResponse;
use crate::state::agent_state::AgentState;
use axum::{
    Json,
    extract::{Extension, Path, State},
};

pub async fn agent(Extension(agent): Extension<Agent>) -> Json<ApiSuccessResponse<AgentReadDto>> {
    Json(ApiSuccessResponse::send(AgentReadDto::from(agent)))
}

pub async fn query(
    State(state): State<AgentState>,
    Path(id): Path<i32>,
) -> Result<Json<AgentReadDto>, ApiError> {
    let agent: Result<Agent, DbError> = state.agent_repo.find(id.try_into().unwrap()).await;
    match agent {
        Ok(agent) => Ok(Json(AgentReadDto::from(agent))),
        Err(e) => Err(ApiError::DbError(e)),
    }
}

pub async fn register(
    State(state): State<AgentState>,
    ValidatedRequest(payload): ValidatedRequest<AgentRegisterDto>,
) -> Result<Json<AgentReadDto>, ApiError> {
    let agent = state.agent_service.create_agent(payload).await;
    match agent {
        Ok(agent) => Ok(Json(agent)),
        Err(e) => Err(e),
    }
}
