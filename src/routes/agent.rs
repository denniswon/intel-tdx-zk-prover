use crate::handler::agent_handler;
use crate::state::agent_state::AgentState;
use axum::{Router, routing::{get, post}};

pub fn routes() -> Router<AgentState> {
    
    Router::new()
        .route("/agent/register", post(agent_handler::register))
        .route("/agent/{id}", get(agent_handler::query))
}
