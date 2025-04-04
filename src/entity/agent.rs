use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ethereum_types::Address;

#[derive(Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct Agent {
    pub id: i32,
    pub agent_name: String,
    pub agent_type: String,
    pub agent_uri: String,
    pub agent_description: String,
    pub agent_owner: Address,
    pub agent_status: AgentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(strum_macros::Display, Debug, Clone, Deserialize, Serialize)]
pub enum AgentStatus {
    Active,
    Inactive,
}
