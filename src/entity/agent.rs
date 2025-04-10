use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::evm::EvmAddress;

#[derive(Clone, Debug, Deserialize, Serialize, FromRow)]
pub struct Agent {
    pub id: i32,
    pub agent_name: String,
    pub agent_type: String,
    pub agent_uri: String,
    pub agent_description: Option<String>,
    pub agent_owner: EvmAddress,
    pub agent_status: AgentStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(strum_macros::Display, Debug, Clone, Deserialize, Serialize, sqlx::Type)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "agent_status", rename_all = "snake_case")]
pub enum AgentStatus {
    Active,
    Inactive,
}
