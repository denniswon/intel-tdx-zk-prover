#![allow(dead_code)]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

#[derive(Clone, Debug, sqlx::FromRow)]
#[sqlx(type_name = "tdx_quote", rename_all = "snake_case")]
pub struct TdxQuote {
    pub id: Uuid,
    pub onchain_request_id: Uuid,
    pub status: TdxQuoteStatus,
    pub quote: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(strum_macros::Display, Debug, Clone, Copy, sqlx::Type, Deserialize, Serialize)]
#[strum(serialize_all = "lowercase")]
#[sqlx(type_name = "status", rename_all = "lowercase")]
pub enum TdxQuoteStatus {
    Pending,
    Failure,
    Success,
}
