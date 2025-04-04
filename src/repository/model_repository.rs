use crate::config::database::{Database, DatabaseTrait};
use crate::entity::request::User;
use async_trait::async_trait;
use sqlx;
use sqlx::Error;
use std::sync::Arc;

#[derive(Clone)]
pub struct ModelRepository {
    pub(crate) db_conn: Arc<Database>,
}

#[async_trait]
pub trait ModelRepositoryTrait {
    fn new(db_conn: &Arc<Database>) -> Self;
    async fn find_all_by_model_creator_address(&self, model_creator_address: String) -> Vec<Model>;
    async fn find_all_active(&self) -> Vec<Model>;
    async fn find(&self, id: u64) -> Result<Model, Error>;
}

#[async_trait]
impl ModelRepositoryTrait for ModelRepository {
    fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: Arc::clone(db_conn),
        }
    }

    async fn find_all_by_model_creator_address(&self, model_creator_address: String) -> Vec<Model> {
        let models = sqlx::query_as::<_, Model>("SELECT * FROM models WHERE model_creator_address = ?")
            .bind(model_creator_address)
            .fetch_all(self.db_conn.get_pool())
            .await
            .unwrap_or(vec![]);
        return models;
    }

    async fn find_all_active(&self) -> Vec<Model> {
        let models = sqlx::query_as::<_, Model>("SELECT * FROM models WHERE is_active = 1")
            .fetch_all(self.db_conn.get_pool())
            .await
            .unwrap_or(vec![]);
        return models;
    }

    async fn find(&self, id: u64) -> Result<Model, Error> {
        let model = sqlx::query_as::<_, Model>("SELECT * FROM models WHERE id = ?")
            .bind(id)
            .fetch_one(self.db_conn.get_pool())
            .await;
        return model;
    }
}
