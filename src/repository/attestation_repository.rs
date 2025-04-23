use crate::config::database::{Database, DatabaseTrait};
use crate::entity::attestation::{Attestation, AttestationType, VerificationStatus};
use crate::entity::quote::Quote;
use async_trait::async_trait;
use sqlx::types::Uuid;
use crate::error::db_error::DbError;
use std::sync::Arc;

#[derive(Clone)]
pub struct AttestationRepository {
    pub(crate) db_conn: Arc<Database>,
}

#[async_trait]
pub trait AttestationRepositoryTrait {
    fn new(db_conn: &Arc<Database>) -> Self;
    async fn find_all_by_attestation_type(
        &self,
        attestation_type: AttestationType,
        verification_status: Option<VerificationStatus>
    ) -> Vec<Attestation>;
    async fn find_all_by_agent_id(&self,
        agent_id: i32,
        attestation_type: Option<AttestationType>,
        verification_status: Option<VerificationStatus>
    ) -> Vec<Attestation>;
    async fn find_all_by_request_id(&self,
        request_id: i32,
        verification_status: Option<VerificationStatus>
    ) -> Vec<Attestation>;
    async fn find(&self, id: u64) -> Result<Attestation, DbError>;
    async fn find_quote(&self, id: Uuid) -> Result<Attestation, DbError>;
}

#[async_trait]
impl AttestationRepositoryTrait for AttestationRepository {
    fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: Arc::clone(db_conn),
        }
    }

    async fn find_all_by_attestation_type(&self,
        attestation_type: AttestationType,
        verification_status: Option<VerificationStatus>
    ) -> Vec<Attestation> {
        match verification_status {
            Some(status) => {
                let attestations = sqlx::query_as::<_, Attestation>("SELECT * FROM attestations WHERE attestation_type = ? AND verification_status = ?")
                    .bind(attestation_type)
                    .bind(status)
                    .fetch_all(self.db_conn.get_pool())
                    .await
                    .unwrap_or(vec![]);
                return attestations;
            }
            None => {
                let attestations = sqlx::query_as::<_, Attestation>("SELECT * FROM attestations WHERE attestation_type = ?")
                    .bind(attestation_type)
                    .fetch_all(self.db_conn.get_pool())
                    .await
                    .unwrap_or(vec![]);
                return attestations;
            }
        }
    }

    async fn find_all_by_agent_id(&self,
        agent_id: i32,
        attestation_type: Option<AttestationType>,
        verification_status: Option<VerificationStatus>
    ) -> Vec<Attestation> {
        let attestation_type = match attestation_type {
            Some(_type) => {
                _type
            }
            None => {
                AttestationType::DcapV3 
            }
        };

        match verification_status {
            Some(status) => {
                let attestations = sqlx::query_as::<_, Attestation>("SELECT * FROM attestations WHERE agent_id = ? AND attestation_type = ? AND verification_status = ?")
                    .bind(agent_id)
                    .bind(attestation_type)
                    .bind(status)
                    .fetch_all(self.db_conn.get_pool())
                    .await
                    .unwrap_or(vec![]);
                return attestations;
            }
            None => {
                let attestations = sqlx::query_as::<_, Attestation>("SELECT * FROM attestations WHERE agent_id = ? AND attestation_type = ?")
                    .bind(agent_id)
                    .bind(attestation_type)
                    .fetch_all(self.db_conn.get_pool())
                    .await
                    .unwrap_or(vec![]);
                return attestations;
            }
        }
    }

    async fn find_all_by_request_id(&self,
        request_id: i32,
        verification_status: Option<VerificationStatus>
    ) -> Vec<Attestation> {
        match verification_status {
            Some(status) => {
                let attestations = sqlx::query_as::<_, Attestation>("SELECT * FROM attestations WHERE request_id = ? AND verification_status = ?")
                    .bind(request_id)
                    .bind(status)
                    .fetch_all(self.db_conn.get_pool())
                    .await
                    .unwrap_or(vec![]);
                return attestations;
            }
            None => {
                let attestations = sqlx::query_as::<_, Attestation>("SELECT * FROM attestations WHERE request_id = ?")
                    .bind(request_id)
                    .fetch_all(self.db_conn.get_pool())
                    .await
                    .unwrap_or(vec![]);
                return attestations;
            }
        }
    }

    async fn find(&self, id: u64) -> Result<Attestation, DbError> {
        let attestation = sqlx::query_as!(
            Attestation,
            r#"SELECT
            id,
            request_id,
            attestation_type as "attestation_type: _",
            verification_status as "verification_status: _",
            attestation_data,
            created_at as "created_at: _"
            FROM attestations WHERE id = $1"#,
            id as i32,
        )
        .fetch_one(self.db_conn.get_pool())
        .await
        .map_err(|_| DbError::SomethingWentWrong("Failed to fetch attestation".to_string()))?;
        return Ok(attestation);
    }

    async fn find_quote(&self, id: Uuid) -> Result<Quote, DbError> {
        let quote = sqlx::query_as!(
            Quote,
            r#"SELECT
            id,
            onchain_request_id,
            quote,
            created_at as "created_at: _",
            updated_at as "updated_at: _",
            FROM tdx_quote WHERE id = $1"#,
            id,
        )
        .fetch_one(self.db_conn.get_pool())
        .await
        .map_err(|_| DbError::SomethingWentWrong("Failed to fetch attestation".to_string()))?;
        return Ok(quote);
    }
}
