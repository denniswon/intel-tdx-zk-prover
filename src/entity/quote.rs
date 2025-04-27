#![allow(dead_code)]
use std::str::FromStr;

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
    pub proof_type: Option<ProofType>,
    pub txn_hash: Option<Vec<u8>>,
    pub request_id: Option<Vec<u8>>,
}

#[derive(strum_macros::Display, Debug, Clone, Copy, sqlx::Type, Serialize, Deserialize)]
#[strum(serialize_all = "lowercase")]
#[sqlx(type_name = "prooftype", rename_all = "lowercase")]
pub enum ProofType {
    Sp1,
    Risc0,
}

impl FromStr for ProofType {
    type Err = String;

    fn from_str(input: &str) -> Result<ProofType, Self::Err> {
        match input {
            "sp1" => Ok(ProofType::Sp1),
            "risc0" => Ok(ProofType::Risc0),
            _ => Err(format!("Unknown proof type: {}", input)),
        }
    }
}

#[derive(strum_macros::Display, Debug, Clone, Copy, sqlx::Type, Serialize, Deserialize)]
#[strum(serialize_all = "lowercase")]
#[sqlx(type_name = "tdxquotestatus", rename_all = "lowercase")]
pub enum TdxQuoteStatus {
    Pending,
    Failure,
    Success,
}

#[derive(strum_macros::Display, Debug, Clone, Copy, sqlx::Type, Serialize, Deserialize)]
#[strum(serialize_all = "lowercase")]
#[sqlx(type_name = "quote_type", rename_all = "lowercase")]
pub enum QuoteType {
    DcapV3,
    DcapV4,
}
