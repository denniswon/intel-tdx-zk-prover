use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::evm::EvmAddress;

#[derive(Clone, Debug, Deserialize, Serialize, FromRow)]
pub struct Request {
    pub id: i32,
    pub agent_id: i32,
    pub from_address: EvmAddress,
    pub request_data: Option<Vec<u8>>,
    pub request_status: RequestStatus,
    pub created_at: NaiveDateTime,
}

#[derive(strum_macros::Display, Debug, Clone, sqlx::Type, Deserialize, Serialize)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "request_status", rename_all = "snake_case")]
pub enum RequestStatus {
    Fulfilled,
    Pending,
    Failed,
}
