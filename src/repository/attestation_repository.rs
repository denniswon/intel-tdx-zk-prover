use crate::config::database::{Database, DatabaseTrait};
use crate::entity::request::{Request, RequestStatus};
use async_trait::async_trait;
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
    async fn find(&self, id: u64) -> Result<Attestation, Error>;
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
        let ra_type = match attestation_type {
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
                let requests = sqlx::query_as::<_, Request>("SELECT * FROM requests WHERE from_address = ?")
                    .bind(from_address)
                    .fetch_all(self.db_conn.get_pool())
                    .await
                    .unwrap_or(vec![]);
                return requests;
            }
        }
    }

    async fn find(&self, id: u64) -> Result<Attestation, DbError> {
        let attestation = sqlx::query_as::<_, Attestation>("SELECT * FROM attestations WHERE id = ?")
            .bind(id)
            .fetch_one(self.db_conn.get_pool())
            .await;
        match attestation {
            Ok(attestation) => Ok(attestation),
            Err(e) => Err(DbError::SomethingWentWrong(e.to_string()))
        }
    }
}
