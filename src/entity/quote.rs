use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Quote {
    pub id: Uuid,
    pub onchain_request_id: Uuid,
    pub status: QuoteStatus,
    pub quote: Vec<u8>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(strum_macros::Display, Debug, Clone, Copy, sqlx::Type, Deserialize, Serialize)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "status", rename_all = "snake_case")]
pub enum QuoteStatus {
    Pending,
    Failure,
    Success,
}
