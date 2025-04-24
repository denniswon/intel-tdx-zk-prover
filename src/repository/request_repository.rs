#[allow(dead_code)]
use crate::config::database::{Database, DatabaseTrait};
use crate::entity::onchain_request::OnchainRequest;
use async_trait::async_trait;
use sqlx::types::Uuid;
use crate::error::db_error::DbError;
use std::sync::Arc;

#[derive(Clone)]
pub struct OnchainRequestRepository {
    pub(crate) db_conn: Arc<Database>,
}

#[async_trait]
pub trait OnchainRequestRepositoryTrait {
    fn new(db_conn: &Arc<Database>) -> Self;
    async fn find_all_by_model_id(&self, model_id: String) -> Vec<OnchainRequest>;
    async fn find(&self, id: Uuid) -> Result<OnchainRequest, DbError>;
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
            println!("Failed to fetch onchain request: {}", e);
            DbError::SomethingWentWrong("Failed to fetch onchain request".to_string())
        })?;
        return Ok(onchain_request);
    }
}
