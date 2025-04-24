use crate::{
    config::database::Database,
    repository::request_repository::{
        OnchainRequestRepository,
        OnchainRequestRepositoryTrait
    }
};
use std::sync::Arc;

#[derive(Clone)]
pub struct RequestService {
    request_repo: OnchainRequestRepository,
    db_conn: Arc<Database>,
}

impl RequestService {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            request_repo: OnchainRequestRepository::new(db_conn),
            db_conn: Arc::clone(db_conn),
        }
    }
}
