use crate::config::database::{Database, DatabaseTrait};
use crate::entity::request::User;
use async_trait::async_trait;
use sqlx;
use sqlx::Error;
use std::sync::Arc;

#[derive(Clone)]
pub struct RequestRepository {
    pub(crate) db_conn: Arc<Database>,
}

#[async_trait]
pub trait RequestRepositoryTrait {
    fn new(db_conn: &Arc<Database>) -> Self;
    async fn find_all_by_from_address(&self, from_address: String) -> Vec<Request>;
    async fn find(&self, id: u64) -> Result<Request, Error>;
}

#[async_trait]
impl RequestRepositoryTrait for RequestRepository {
    fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: Arc::clone(db_conn),
        }
    }

    async fn find_all_by_from_address(&self, from_address: String) -> Vec<Request> {
        let requests = sqlx::query_as::<_, Request>("SELECT * FROM requests WHERE from_address = ?")
            .bind(from_address)
            .fetch_all(self.db_conn.get_pool())
            .await
            .unwrap_or(vec![]);
        return requests;
    }

    async fn find(&self, id: u64) -> Result<Request, Error> {
        let request = sqlx::query_as::<_, Request>("SELECT * FROM requests WHERE id = ?")
            .bind(id)
            .fetch_one(self.db_conn.get_pool())
            .await;
        return request;
    }
}
