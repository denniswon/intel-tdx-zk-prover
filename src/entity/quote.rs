#![allow(dead_code)]
use chrono::{DateTime, Utc};
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
    pub proof_type: Option<ProofType>,
    pub txn_hash: Option<Vec<u8>>,
    pub request_id: Option<Vec<u8>>,
}

#[derive(strum_macros::Display, Debug, Clone, Copy, sqlx::Type)]
#[strum(serialize_all = "lowercase")]
#[sqlx(type_name = "prooftype", rename_all = "lowercase")]
pub enum ProofType {
    Sp1,
    Risc0,
}

#[derive(strum_macros::Display, Debug, Clone, Copy, sqlx::Type)]
#[strum(serialize_all = "lowercase")]
#[sqlx(type_name = "tdxquotestatus", rename_all = "lowercase")]
pub enum TdxQuoteStatus {
    Pending,
    Failure,
    Success,
}
