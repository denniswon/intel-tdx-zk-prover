use crate::dto::agent_dto::{AgentReadDto, AgentRegisterDto};
use crate::entity::agent::Agent;
use crate::error::{api_error::ApiError, api_request_error::ValidatedRequest};
use crate::state::agent_state::AgentState;
use axum::response::IntoResponse;
use axum::{Json, extract::{Extension, Path, State}};
use crate::response::api_response::ApiSuccessResponse;
use crate::service::agent_service::AgentService;
use sqlx::Error;

pub async fn agent(
    Extension(agent): Extension<Agent>,
) -> Json<ApiSuccessResponse<AgentReadDto>> {
    Json(ApiSuccessResponse::send(AgentReadDto::from(agent)))
}

pub async fn query(
    State(state): State<AgentState>,
    Path(id): Path<i32>
) -> Result<Json<AgentReadDto>, ApiError> {
    let agent: Result<Agent, DbError> = state.agent_repo.find(id).await;
    return match agent {
        Ok(agent) => Ok(Json(AgentReadDto::from(agent))),
        Err(e) => Err(Json(e.into_response()))
    };
}

pub async fn register(
    State(state): State<AgentState>,
    ValidatedRequest(payload): ValidatedRequest<AgentRegisterDto>,
) -> Result<Json<AgentReadDto>, ApiError> {
    let agent = state.agent_service.create_agent(payload).await;
    return match agent {
        Ok(agent) => Ok(Json(ApiSuccessResponse::send(agent))),
        Err(e) => Err(e.into_response())
    };
}
