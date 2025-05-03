#[allow(dead_code)]
use crate::config::database::{Database, DatabaseTrait};
use crate::entity::{request::OnchainRequest, quote::TdxQuoteStatus};
use async_trait::async_trait;
use sqlx::{types::Uuid, FromRow};
use crate::error::db_error::DbError;
use std::sync::Arc;

#[derive(Debug, FromRow)]
pub struct OnchainRequestId {
    pub request_id: Vec<u8>,
}

impl OnchainRequestId {
    pub fn new(request_id: Vec<u8>) -> Self {
        Self { request_id }
    }
}

#[derive(Clone)]
pub struct OnchainRequestRepository {
    pub(crate) db_conn: Arc<Database>,
}

#[async_trait]
pub trait OnchainRequestRepositoryTrait {
    fn new(db_conn: &Arc<Database>) -> Self;
    async fn find_all_by_model_id(&self, model_id: String) -> Vec<OnchainRequest>;
    async fn find(&self, id: Uuid) -> Result<OnchainRequest, DbError>;
    async fn find_by_request_id(&self, request_id: Vec<u8>) -> Result<OnchainRequest, DbError>;
    async fn find_request_ids_by_status(&self, status: Option<TdxQuoteStatus>, max_count: Option<i64>) -> Vec<OnchainRequestId>;
}

#[async_trait]
impl OnchainRequestRepositoryTrait for OnchainRequestRepository {
    fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: Arc::clone(db_conn),
        }
    }

    async fn find_all_by_model_id(&self, model_id: String) -> Vec<OnchainRequest> {
        let onchain_requests = sqlx::query_as!(
            OnchainRequest,
            r#"SELECT
            id,
            creator_address,
            operator_address,
            model_id,
            fee_wei,
            nonce,
            request_id,
            deadline,
            is_cancelled,
            cancelled_at,
            created_at as "created_at: _",
            updated_at as "updated_at: _"
            FROM onchain_request WHERE model_id = $1"#,
            model_id
        ).fetch_all(self.db_conn.get_pool())
        .await
        .unwrap_or(vec![]);
        return onchain_requests;
    }

    async fn find(&self, id: Uuid) -> Result<OnchainRequest, DbError> {
        let onchain_request = sqlx::query_as!(
            OnchainRequest,
            r#"SELECT
            id,
            creator_address,
            operator_address,
            model_id,
            fee_wei,
            nonce,
            request_id,
            deadline,
            is_cancelled,
            cancelled_at,
            created_at as "created_at: _",
            updated_at as "updated_at: _"
            FROM onchain_request WHERE id = $1"#,
            id
        ).fetch_one(self.db_conn.get_pool())
        .await
        .map_err(|e| {
            tracing::info!("Failed to fetch onchain request: {}", e);
            DbError::SomethingWentWrong("Failed to fetch onchain request".to_string())
        })?;
        return Ok(onchain_request);
    }

    async fn find_by_request_id(&self, request_id: Vec<u8>) -> Result<OnchainRequest, DbError> {
        let onchain_request = sqlx::query_as!(
            OnchainRequest,
            r#"SELECT
            id,
            creator_address,
            operator_address,
            model_id,
            fee_wei,
            nonce,
            request_id,
            deadline,
            is_cancelled,
            cancelled_at,
            created_at as "created_at: _",
            updated_at as "updated_at: _"
            FROM onchain_request WHERE request_id = $1"#,
            request_id
        ).fetch_one(self.db_conn.get_pool())
        .await
        .map_err(|e| {
            tracing::info!("Failed to fetch onchain request: {}", e);
            DbError::SomethingWentWrong("Failed to fetch onchain request".to_string())
        })?;
        return Ok(onchain_request);
    }

    async fn find_request_ids_by_status(&self, status: Option<TdxQuoteStatus>, max_count: Option<i64>) -> Vec<OnchainRequestId> {
        match status {
            Some(status) => {
                match max_count {
                    Some(count) => sqlx::query_as::<_, OnchainRequestId>(
                        r#"SELECT onchain_request.request_id
                        FROM onchain_request
                        JOIN tdx_quote
                        ON onchain_request.id = tdx_quote.onchain_request_id
                        WHERE tdx_quote.status = ?
                        ORDER BY tdx_quote.created_at DESC
                        LIMIT ?"#)
                        .bind(status)
                        .bind(count)
                        .fetch_all(self.db_conn.get_pool())
                        .await
                        .unwrap_or(vec![]),
                    None => sqlx::query_as::<_, OnchainRequestId>(
                        r#"SELECT onchain_request.request_id
                        FROM onchain_request
                        JOIN tdx_quote
                        ON onchain_request.id = tdx_quote.onchain_request_id
                        WHERE tdx_quote.status = ?
                        ORDER BY tdx_quote.created_at DESC"#)
                        .bind(status)
                        .fetch_all(self.db_conn.get_pool())
                        .await
                        .unwrap_or(vec![]),
                }
            }
            None => {
                match max_count {
                    Some(count) => sqlx::query_as::<_, OnchainRequestId>(
                        r#"SELECT onchain_request.request_id
                        FROM onchain_request
                        JOIN tdx_quote
                        ON onchain_request.id = tdx_quote.onchain_request_id
                        ORDER BY tdx_quote.created_at DESC
                        LIMIT ?"#)
                        .bind(count)
                        .fetch_all(self.db_conn.get_pool())
                        .await
                        .unwrap_or(vec![]),
                    None => sqlx::query_as::<_, OnchainRequestId>(
                        r#"SELECT onchain_request.request_id
                        FROM onchain_request
                        JOIN tdx_quote
                        ON onchain_request.id = tdx_quote.onchain_request_id
                        ORDER BY tdx_quote.created_at DESC"#)
                        .fetch_all(self.db_conn.get_pool())
                        .await
                        .unwrap_or(vec![]),
                }
            }
        }
    }
}
