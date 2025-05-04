#[allow(dead_code)]
use crate::config::database::{Database, DatabaseTrait};
use crate::entity::quote::{ProofType, TdxQuote, TdxQuoteStatus};
use async_trait::async_trait;
use sqlx::types::Uuid;
use crate::error::db_error::DbError;
use std::sync::Arc;
use crate::repository::request_repository::OnchainRequestId;

#[derive(Clone)]
pub struct QuoteRepository {
    pub(crate) db_conn: Arc<Database>,
}

#[async_trait]
pub trait QuoteRepositoryTrait {
    fn new(db_conn: &Arc<Database>) -> Self;
    async fn find(&self, id: Uuid) -> Result<TdxQuote, DbError>;
    async fn find_all_by_onchain_request_id(&self,
        onchain_request_id: Uuid,
        verification_status: Option<TdxQuoteStatus>
    ) -> Vec<TdxQuote>;
    async fn find_by_onchain_request_id(&self, onchain_request_id: Uuid) -> Result<TdxQuote, DbError>;
    async fn update_status(
        &self,
        id: Uuid,
        proof_type: ProofType,
        status: TdxQuoteStatus,
        transaction_hash: Option<Vec<u8>>,
        prover_request_id: Option<Vec<u8>>
    ) -> Result<(), DbError>;
    async fn find_request_ids_by_status(
        &self,
        status: Option<TdxQuoteStatus>,
        max_count: Option<i64>
    ) -> Vec<OnchainRequestId>;
}

#[async_trait]
impl QuoteRepositoryTrait for QuoteRepository {
    fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: Arc::clone(db_conn),
        }
    }

    async fn find(&self, id: Uuid) -> Result<TdxQuote, DbError> {
        let quote = sqlx::query_as!(
            TdxQuote,
            r#"SELECT
            id,
            proof_type as "proof_type: crate::entity::quote::ProofType",
            request_id,
            txn_hash,
            onchain_request_id,
            quote,
            created_at as "created_at: _",
            updated_at as "updated_at: _",
            status as "status: crate::entity::quote::TdxQuoteStatus"
            FROM tdx_quote WHERE id = $1"#,
            id,
        )
        .fetch_one(self.db_conn.get_pool())
        .await
        .map_err(|e| {
            tracing::info!("Failed to fetch quote: {}", e);
            DbError::SomethingWentWrong("Failed to fetch quote".to_string())
        })?;
        return Ok(quote);
    }

    async fn find_all_by_onchain_request_id(&self,
        onchain_request_id: Uuid,
        verification_status: Option<TdxQuoteStatus>
    ) -> Vec<TdxQuote> {
        match verification_status {
            Some(status) => {
                let quotes = sqlx::query_as::<_, TdxQuote>("SELECT * FROM tdx_quote WHERE onchain_request_id = ? AND status = ?")
                    .bind(onchain_request_id)
                    .bind(status)
                    .fetch_all(self.db_conn.get_pool())
                    .await
                    .unwrap_or(vec![]);
                return quotes;
            }
            None => {
                let quotes = sqlx::query_as::<_, TdxQuote>("SELECT * FROM tdx_quote WHERE onchain_request_id = ?")
                    .bind(onchain_request_id)
                    .fetch_all(self.db_conn.get_pool())
                    .await
                    .unwrap_or(vec![]);
                return quotes;
            }
        }
    }

    async fn find_by_onchain_request_id(&self, onchain_request_id: Uuid) -> Result<TdxQuote, DbError> {
        let quote = sqlx::query_as!(
            TdxQuote,
            r#"SELECT
            id,
            proof_type as "proof_type: crate::entity::quote::ProofType",
            request_id,
            txn_hash,
            onchain_request_id,
            quote,
            created_at as "created_at: _",
            updated_at as "updated_at: _",
            status as "status: crate::entity::quote::TdxQuoteStatus"
            FROM tdx_quote WHERE onchain_request_id = $1"#,
            onchain_request_id,
        )
        .fetch_one(self.db_conn.get_pool())
        .await
        .map_err(|e| {
            tracing::info!("Failed to fetch quote: {}", e);
            DbError::SomethingWentWrong("Failed to fetch quote".to_string())
        })?;
        return Ok(quote);
    }

    async fn update_status(
        &self,
        id: Uuid,
        proof_type: ProofType,
        status: TdxQuoteStatus,
        transaction_hash: Option<Vec<u8>>,
        _prover_request_id: Option<Vec<u8>>
    ) -> Result<(), DbError> {
        tracing::debug!("Updating quote status for id: {} with status: {}", id, status);
        match transaction_hash {
            Some(txn_hash) => {
                sqlx::query!(
                    r#"UPDATE tdx_quote SET status = $2, txn_hash = $3, proof_type = $4 WHERE id = $1"#,
                    id,
                    status as TdxQuoteStatus,
                    txn_hash,
                    proof_type as ProofType,
                )
                .execute(self.db_conn.get_pool())
                .await
                .map_err(|e| {
                    tracing::info!("Failed to update quote status: {}", e);
                    DbError::SomethingWentWrong("Failed to update quote status".to_string())
                })?;
            }
            None => {
                sqlx::query!(
                    r#"UPDATE tdx_quote SET status = $2, proof_type = $3 WHERE id = $1"#,
                    id,
                    status as TdxQuoteStatus,
                    proof_type as ProofType,
                )
                .execute(self.db_conn.get_pool())
                .await
                .map_err(|e| {
                    tracing::info!("Failed to update quote status: {}", e);
                    DbError::SomethingWentWrong("Failed to update quote status".to_string())
                })?;
            }
        }
        Ok(())
    }

    async fn find_request_ids_by_status(
        &self,
        status: Option<TdxQuoteStatus>,
        max_count: Option<i64>
    ) -> Vec<OnchainRequestId> {
        match status {
            Some(status) => {
                let quotes = match max_count {
                    Some(count) => sqlx::query_as::<_, OnchainRequestId>(
                            r#"SELECT onchain_request_id FROM tdx_quote WHERE status = ? LIMIT ?"#
                        )
                        .bind(status)
                        .bind(count)
                        .fetch_all(self.db_conn.get_pool())
                        .await
                        .unwrap_or(vec![]),
                    None => sqlx::query_as::<_, OnchainRequestId>(
                            r#"SELECT onchain_request_id FROM tdx_quote WHERE status = ?"#
                        )
                        .bind(status)
                        .fetch_all(self.db_conn.get_pool())
                        .await
                        .unwrap_or(vec![]),
                };
                return quotes;
            },
            None => {
                let quotes = match max_count {
                    Some(count) => sqlx::query_as::<_, OnchainRequestId>(
                            r#"SELECT onchain_request_id FROM tdx_quote LIMIT ? ORDER BY created_at DESC"#
                        )
                        .bind(count)
                        .fetch_all(self.db_conn.get_pool())
                        .await
                        .unwrap_or(vec![]),
                    None => sqlx::query_as::<_, OnchainRequestId>(
                            r#"SELECT onchain_request_id FROM tdx_quote ORDER BY created_at DESC"#
                        )
                        .fetch_all(self.db_conn.get_pool())
                        .await
                        .unwrap_or(vec![]),
                };
                return quotes;
            },
        }
    }
}
