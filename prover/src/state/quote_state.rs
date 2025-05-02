#![allow(dead_code)]
use crate::config::database::Database;
use crate::repository::quote_repository::{QuoteRepository, QuoteRepositoryTrait};
use crate::service::quote_service::QuoteService;
use std::sync::Arc;

#[derive(Clone)]
pub struct QuoteState {
    pub quote_service: QuoteService,
    pub quote_repo: QuoteRepository,
}

impl QuoteState {
    pub fn new(db_conn: &Arc<Database>) -> QuoteState {
        Self {
            quote_service: QuoteService::new(db_conn),
            quote_repo: QuoteRepository::new(db_conn),
        }
    }
}
