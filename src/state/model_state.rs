use crate::config::database::Database;
use crate::repository::model_repository;
use crate::service::model_service::ModelService;
use std::sync::Arc;

#[derive(Clone)]
pub struct ModelState {
    pub(crate) model_repo: model_repository::ModelRepository,
    pub(crate) model_service: ModelService,
}

impl ModelState {
    pub fn new(db_conn: &Arc<Database>) -> ModelState {
        Self {
            model_service: ModelService::new(db_conn),
            model_repo: model_repository::ModelRepository::new(db_conn),
        }
    }
}
