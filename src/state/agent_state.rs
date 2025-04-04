use crate::config::database::Database;
use crate::repository::agent_repository;
use crate::service::agent_service::AgentService;
use std::sync::Arc;

#[derive(Clone)]
pub struct AgentState {
    pub(crate) agent_repo: agent_repository::AgentRepository,
    pub(crate) agent_service: AgentService,
}

impl AgentState {
    pub fn new(db_conn: &Arc<Database>) -> AgentState {
        Self {
            agent_service: AgentService::new(db_conn),
            agent_repo: agent_repository::AgentRepositoryTrait::new(db_conn),
        }
    }
}
