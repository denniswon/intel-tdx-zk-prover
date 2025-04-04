
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::evm::EvmAddress;

#[derive(Clone, Deserialize, Serialize, FromRow)]
pub struct Agent {
    pub id: i32,
    pub agent_name: String,
    pub agent_type: String,
    pub agent_uri: String,
    pub agent_description: String,
    pub agent_owner: EvmAddress,
    pub agent_status: AgentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(strum_macros::Display, Debug, Clone, Deserialize, Serialize, sqlx::Type)]
#[sqlx(type_name = "agent_status")]
pub enum AgentStatus {
    Active,
    Inactive,
}
