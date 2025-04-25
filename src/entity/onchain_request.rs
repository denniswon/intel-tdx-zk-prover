#![allow(dead_code)]
use chrono::{DateTime, Utc};
use sqlx::types::Uuid;

#[derive(Clone, sqlx::FromRow)]
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

impl std::fmt::Debug for OnchainRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OnchainRequest")
            .field("id", &self.id)
            .field("creator_address", &self.creator_address)
            .field("operator_address", &self.operator_address)
            .field("model_id", &self.model_id)
            .field("fee_wei", &self.fee_wei)
            .field("nonce", &self.nonce)
            .field("request_id", &hex::encode(self.request_id.clone()))
            .field("deadline", &self.deadline)
            .field("is_cancelled", &self.is_cancelled)
            .field("cancelled_at", &self.cancelled_at)
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}
