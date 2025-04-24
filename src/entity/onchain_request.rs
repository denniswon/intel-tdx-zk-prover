#![allow(dead_code)]
use chrono::{DateTime, Utc};
use sqlx::types::Uuid;

#[derive(Clone, Debug, sqlx::FromRow)]
#[sqlx(type_name = "onchain_request", rename_all = "snake_case")]
pub struct OnchainRequest {
    pub id: Uuid,
    pub creator_address: String,
    pub operator_address: String,
    pub model_id: String,
    pub fee_wei: i64,
    pub nonce: i64,
    pub request_id: Vec<u8>,
    pub deadline: DateTime<Utc>,
    pub is_cancelled: bool,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
