#![allow(dead_code)]
use crate::config::database::Database;
use crate::repository::request_repository::{OnchainRequestRepository, OnchainRequestRepositoryTrait};
use crate::service::request_service::RequestService;
use std::sync::Arc;

#[derive(Clone)]
pub struct RequestState {
    #[allow(dead_code)]
    pub request_service: RequestService,
    pub request_repo: OnchainRequestRepository,
}

impl RequestState {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            request_service: RequestService::new(db_conn),
            request_repo: OnchainRequestRepository::new(db_conn),
        }
    }
}
