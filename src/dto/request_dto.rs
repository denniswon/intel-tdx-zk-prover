#![allow(dead_code)]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use validator::Validate;

use crate::entity::request::OnchainRequest;

#[derive(Clone, Validate)]
pub struct RequestDto {
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

#[derive(Clone, Serialize, Deserialize)]
pub struct RequestReadDto {
    pub id: String,
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

impl RequestReadDto {
    pub fn from(request: OnchainRequest) -> RequestReadDto {
        Self {
            id: request.id.to_string(),
            creator_address: request.creator_address,
            operator_address: request.operator_address,
            model_id: request.model_id,
            fee_wei: request.fee_wei,
            nonce: request.nonce,
            request_id: request.request_id,
            deadline: request.deadline,
            is_cancelled: request.is_cancelled,
            cancelled_at: request.cancelled_at,
            created_at: request.created_at,
            updated_at: request.updated_at,
        }
    }
}

impl std::fmt::Debug for RequestReadDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequestReadDto")
            .field("id", &self.id)
            .field("creator_address", &self.creator_address)
            .field("operator_address", &self.operator_address)
            .field("model_id", &self.model_id)
            .field("fee_wei", &self.fee_wei)
            .field("nonce", &self.nonce)
            .field("request_id", &self.request_id)
            .field("deadline", &self.deadline)
            .field("is_cancelled", &self.is_cancelled)
            .field("cancelled_at", &self.cancelled_at)
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}
