use crate::config::database::Database;
use crate::repository::attestation_repository;
use crate::service::attestation_service::AttestationService;
use std::sync::Arc;

#[derive(Clone)]
pub struct AttestationState {
    pub(crate) attestation_repo: attestation_repository::AttestationRepository,
    pub(crate) attestation_service: AttestationService,
}

impl AttestationState {
    pub fn new(db_conn: &Arc<Database>) -> AttestationState {
        Self {
            attestation_service: AttestationService::new(db_conn),
            attestation_repo: attestation_repository::AttestationRepositoryTrait::new(db_conn),
        }
    }
}
