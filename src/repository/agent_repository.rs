use crate::config::database::{Database, DatabaseTrait};
use crate::entity::agent::{Agent, AgentStatus};
use crate::error::db_error::DbError;
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Clone)]
pub struct AgentRepository {
    pub(crate) db_conn: Arc<Database>,
}

#[async_trait]
pub trait AgentRepositoryTrait {
    fn new(db_conn: &Arc<Database>) -> Self;
    async fn find_all_by_agent_owner(
        &self,
        agent_owner: String,
        agent_status: Option<AgentStatus>,
    ) -> Vec<Agent>;
    async fn find_by_status(&self, agent_status: AgentStatus) -> Vec<Agent>;
    async fn find(&self, id: u64) -> Result<Agent, DbError>;
}

#[async_trait]
impl AgentRepositoryTrait for AgentRepository {
    fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: Arc::clone(db_conn),
        }
    }

    async fn find_all_by_agent_owner(
        &self,
        agent_owner: String,
        agent_status: Option<AgentStatus>,
    ) -> Vec<Agent> {
        match agent_status {
            Some(status) => {
                let agents = sqlx::query_as::<_, Agent>(
                    "SELECT * FROM agents WHERE agent_owner = ? AND agent_status = ?",
                )
                .bind(agent_owner)
                .bind(status)
                .fetch_all(self.db_conn.get_pool())
                .await
                .unwrap_or(vec![]);
                return agents;
            }
            None => {
                let agents =
                    sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE agent_owner = ?")
                        .bind(agent_owner)
                        .fetch_all(self.db_conn.get_pool())
                        .await
                        .unwrap_or(vec![]);
                return agents;
            }
        }
    }

    async fn find_by_status(&self, agent_status: AgentStatus) -> Vec<Agent> {
        let agents = sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE agent_status = ?")
            .bind(agent_status)
            .fetch_all(self.db_conn.get_pool())
            .await
            .unwrap_or(vec![]);
        return agents;
    }

    async fn find(&self, id: u64) -> Result<Agent, DbError> {
        let agent = sqlx::query_as!(
            Agent,
            r#"SELECT
            id,
            agent_name,
            agent_type,
            agent_uri,
            agent_description,
            agent_owner,
            agent_status as "agent_status: _",
            created_at,
            updated_at
            FROM agents WHERE id = $1"#,
            id as i32
        ).fetch_one(self.db_conn.get_pool()).await;
        match agent {
            Ok(agent) => Ok(agent),
            Err(e) => Err(DbError::SomethingWentWrong(e.to_string())),
        }
    }
}
