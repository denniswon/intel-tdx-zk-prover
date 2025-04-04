use crate::config::database::Database;
use crate::repository::request_repository;
use crate::repository::request_repository::UserRepositoryTrait;
use crate::service::token_service::{TokenService, TokenServiceTrait};
use crate::service::request_service::UserService;
use std::sync::Arc;

#[derive(Clone)]
pub struct AuthState {
    pub(crate) token_service: TokenService,
    pub(crate) user_repo: request_repository::UserRepository,
    pub(crate) user_service: UserService,
}

impl AuthState {
    pub fn new(db_conn: &Arc<Database>) -> AuthState {
        Self {
            token_service: TokenService::new(),
            user_service: UserService::new(db_conn),
            user_repo: request_repository::UserRepository::new(db_conn),
        }
    }
}
