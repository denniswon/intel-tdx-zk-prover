#![allow(dead_code)]
use crate::entity::quote::{TdxQuote, TdxQuoteStatus};
use chrono::{DateTime, Utc};
use sqlx::types::Uuid;
use validator::Validate;

#[derive(Clone, Validate)]
pub struct QuoteDto {
    pub id: Uuid,
    pub onchain_request_id: Uuid,
    pub quote: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: TdxQuoteStatus,
}

#[derive(Clone)]
pub struct QuoteReadDto {
    pub id: Uuid,
    pub onchain_request_id: Uuid,
    pub quote: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: TdxQuoteStatus,
}

impl QuoteReadDto {
    pub fn from(quote: TdxQuote) -> QuoteReadDto {
        Self {
            id: quote.id,
            onchain_request_id: quote.onchain_request_id,
            quote: quote.quote,
            created_at: quote.created_at,
            updated_at: quote.updated_at,
            status: quote.status,
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
            .finish()
    }
}
