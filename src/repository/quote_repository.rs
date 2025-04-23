#[allow(dead_code)]
use crate::config::database::{Database, DatabaseTrait};
use crate::entity::quote::{TdxQuote, TdxQuoteStatus};
use async_trait::async_trait;
use sqlx::types::Uuid;
use crate::error::db_error::DbError;
use std::sync::Arc;

#[derive(Clone)]
pub struct QuoteRepository {
    pub(crate) db_conn: Arc<Database>,
}

#[async_trait]
pub trait QuoteRepositoryTrait {
    fn new(db_conn: &Arc<Database>) -> Self;
    async fn find_all_by_onchain_request_id(&self,
        onchain_request_id: Uuid,
        verification_status: Option<TdxQuoteStatus>
    ) -> Vec<TdxQuote>;
    async fn find_by_onchain_request_id(&self, onchain_request_id: Uuid) -> Result<TdxQuote, DbError>;
}

#[async_trait]
impl QuoteRepositoryTrait for QuoteRepository {
    fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: Arc::clone(db_conn),
        }
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
            println!("Failed to fetch quote: {}", e);
            DbError::SomethingWentWrong("Failed to fetch quote".to_string())
        })?;
        return Ok(quote);
    }
}
