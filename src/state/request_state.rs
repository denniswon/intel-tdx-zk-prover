use crate::config::database::Database;
use crate::repository::request_repository::{RequestRepository, RequestRepositoryTrait};
use crate::service::request_service::RequestService;
use std::sync::Arc;

#[derive(Clone)]
pub struct RequestState {
    pub request_service: RequestService,
    pub request_repo: RequestRepository,
}

impl RequestState {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            request_service: RequestService::new(db_conn),
            request_repo: RequestRepository::new(db_conn),
        }
    }
}
