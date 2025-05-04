#![allow(dead_code)]
use crate::entity::quote::{ProofType, TdxQuote, TdxQuoteStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use validator::Validate;

#[derive(Clone, Validate)]
pub struct QuoteDto {
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

#[derive(Clone, Serialize, Deserialize)]
pub struct QuoteReadDto {
    pub id: String,
    pub onchain_request_id: String,
    pub status: TdxQuoteStatus,
    pub quote: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub proof_type: Option<ProofType>,
    pub txn_hash: Option<Vec<u8>>,
    pub request_id: Option<Vec<u8>>,
}

impl QuoteReadDto {
    pub fn from(quote: TdxQuote) -> QuoteReadDto {
        Self {
            id: quote.id.to_string(),
            onchain_request_id: quote.onchain_request_id.to_string(),
            quote: quote.quote,
            created_at: quote.created_at,
            updated_at: quote.updated_at,
            status: quote.status,
            proof_type: quote.proof_type,
            txn_hash: quote.txn_hash,
            request_id: quote.request_id,
        }
    }
}

impl std::fmt::Debug for QuoteReadDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuoteReadDto")
            .field("id", &self.id)
            .field("onchain_request_id", &self.onchain_request_id)
            .field("quote", &self.quote)
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .field("status", &self.status)
            .field("proof_type", &self.proof_type)
            .field("txn_hash", &self.txn_hash)
            .field("request_id", &self.request_id)
            .finish()
    }
}
#[derive(Clone, Validate, Serialize, Deserialize)]
pub struct QuoteRegisterDto {
    pub proof_type: Option<ProofType>,
    pub onchain_request_id: String,
    pub quote: Vec<u8>,
    pub txn_hash: Option<Vec<u8>>,
    pub request_id: Option<Vec<u8>>,
    pub status: TdxQuoteStatus,
}

impl QuoteRegisterDto {
    pub fn from(quote: TdxQuote) -> QuoteRegisterDto {
        Self {
            onchain_request_id: quote.onchain_request_id.to_string(),
            quote: quote.quote,
            proof_type: quote.proof_type,
            txn_hash: quote.txn_hash,
            request_id: quote.request_id,
            status: quote.status,
        }
    }
}

impl std::fmt::Debug for QuoteRegisterDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuoteRegisterDto")
            .field("onchain_request_id", &self.onchain_request_id)
            .field("quote", &self.quote)
            .field("status", &self.status)
            .field("proof_type", &self.proof_type)
            .field("txn_hash", &self.txn_hash)
            .field("request_id", &self.request_id)
            .finish()
    }
}
