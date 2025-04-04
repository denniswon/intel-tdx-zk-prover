use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::evm::{EvmAddress, WeiAmount};

#[derive(Clone, Deserialize, Serialize, FromRow)]
pub struct Request {
    pub id: i32,
    pub agent_id: i32,
    pub from_address: EvmAddress,
    pub prompt: String,
    pub request_data: Option<Vec<u8>>,
    pub fee_amount: WeiAmount,
    pub request_status: RequestStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(strum_macros::Display, Debug, Clone, sqlx::Type, Deserialize, Serialize)]
pub enum RequestStatus {
    Fulfilled,
    Pending,
    Failed,
}
