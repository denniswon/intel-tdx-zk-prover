use crate::config::database::{Database, DatabaseTrait};
use crate::dto::agent_dto::{AgentReadDto, AgentRegisterDto};
use crate::entity::agent::Agent;
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::repository::agent_repository::{AgentRepository, AgentRepositoryTrait};
use sqlx::Error as SqlxError;
use std::sync::Arc;

#[derive(Clone)]
pub struct AgentService {
    agent_repo: AgentRepository,
    db_conn: Arc<Database>,
}

impl AgentService {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            agent_repo: AgentRepository::new(db_conn),
            db_conn: Arc::clone(db_conn),
        }
    }

    pub async fn create_agent(&self, payload: AgentRegisterDto) -> Result<AgentReadDto, ApiError> {
        let agent = self.add_agent(payload).await;

        return match agent {
            Ok(agent) => Ok(AgentReadDto::from(agent)),
            Err(e) => match e {
                SqlxError::Database(e) => match e.code() {
                    Some(code) => {
                        if code == "23000" {
                            Err(DbError::UniqueConstraintViolation(e.to_string()))?
                        } else {
                            Err(DbError::SomethingWentWrong(e.to_string()))?
                        }
                    }
                    _ => Err(DbError::SomethingWentWrong(e.to_string()))?,
                },
                _ => Err(DbError::SomethingWentWrong(e.to_string()))?,
            },
        };
    }

    async fn add_agent(&self, payload: AgentRegisterDto) -> Result<Agent, SqlxError> {
        let insert = sqlx::query_as!(
            Agent,
            r#"
                INSERT INTO agents (agent_owner, agent_name, agent_description, agent_type, agent_uri, agent_status)
                VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            payload.agent_owner,
            payload.agent_name,
            payload.agent_description,
            payload.agent_type,
            payload.agent_uri,
            payload.agent_status
        )
        .execute(self.db_conn.get_pool())
        .await?;

        let agent = self.agent_repo.find(insert.last_insert_id()).await?;
        return Ok(agent);
    }
}
