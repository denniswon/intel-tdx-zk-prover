use crate::entity::{
    agent::{Agent, AgentStatus},
    evm::EvmAddress,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct AgentRegisterDto {
    #[validate(length(min = 1, message = "Agent name cannot be empty"))]
    pub agent_name: String,
    #[validate(length(min = 1, message = "Agent type cannot be empty"))]
    pub agent_type: String,
    #[validate(length(min = 1, message = "Agent URI cannot be empty"))]
    pub agent_uri: String,
    pub agent_description: String,
    pub agent_owner: EvmAddress,
    pub agent_status: AgentStatus,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct AgentReadDto {
    pub id: i32,
    pub agent_name: String,
    pub agent_type: String,
    pub agent_uri: String,
    pub agent_description: Option<String>,
    pub agent_owner: EvmAddress,
    pub agent_status: AgentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl AgentReadDto {
    pub fn from(agent: Agent) -> AgentReadDto {
        Self {
            id: agent.id,
            agent_name: agent.agent_name,
            agent_type: agent.agent_type,
            agent_uri: agent.agent_uri,
            agent_description: agent.agent_description,
            agent_owner: agent.agent_owner,
            agent_status: agent.agent_status,
            created_at: agent.created_at,
            updated_at: agent.updated_at,
        }
    }
}

impl std::fmt::Debug for AgentReadDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentReadDto")
            .field("id", &self.id)
            .field("agent_name", &self.agent_name)
            .field("agent_type", &self.agent_type)
            .field("agent_uri", &self.agent_uri)
            .field("agent_description", &self.agent_description)
            .field("agent_owner", &self.agent_owner)
            .field("agent_status", &self.agent_status)
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}

impl std::fmt::Debug for AgentRegisterDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentRegisterDto")
            .field("agent_name", &self.agent_name)
            .field("agent_type", &self.agent_type)
            .field("agent_uri", &self.agent_uri)
            .field("agent_description", &self.agent_description)
            .field("agent_owner", &self.agent_owner)
            .field("agent_status", &self.agent_status)
            .finish()
    }
}
