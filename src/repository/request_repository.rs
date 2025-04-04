use crate::config::database::{Database, DatabaseTrait};
use crate::entity::request::{Request, RequestStatus};
use async_trait::async_trait;
use crate::error::db_error::DbError;
use std::sync::Arc;

#[derive(Clone)]
pub struct RequestRepository {
    pub(crate) db_conn: Arc<Database>,
}

#[async_trait]
pub trait RequestRepositoryTrait {
    fn new(db_conn: &Arc<Database>) -> Self;
    async fn find_all_by_from_address(&self, from_address: String, request_status: Option<RequestStatus>) -> Vec<Request>;
    async fn find_all_by_status(&self, request_status: RequestStatus) -> Vec<Request>;
    async fn find_all_by_agent_id(&self, agent_id: i32, request_status: Option<RequestStatus>) -> Vec<Request>;
    async fn find(&self, id: u64) -> Result<Request, Error>;
}

#[async_trait]
impl RequestRepositoryTrait for RequestRepository {
    fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: Arc::clone(db_conn),
        }
    }

    async fn find_all_by_from_address(&self, from_address: String, request_status: Option<RequestStatus>) -> Vec<Request> {
        match request_status {
            Some(status) => {
                let requests = sqlx::query_as::<_, Request>("SELECT * FROM requests WHERE from_address = ? AND request_status = ?")
                    .bind(from_address)
                    .bind(status)
                    .fetch_all(self.db_conn.get_pool())
                    .await
                    .unwrap_or(vec![]);
                return requests;
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

    async fn find_all_by_agent_id(&self, agent_id: i32, request_status: Option<RequestStatus>) -> Vec<Request> {
        match request_status {
            Some(status) => {
                let requests = sqlx::query_as::<_, Request>("SELECT * FROM requests WHERE agent_id = ? AND request_status = ?")
                    .bind(agent_id)
                    .bind(status)
                    .fetch_all(self.db_conn.get_pool())
                    .await
                    .unwrap_or(vec![]);
                return requests;
            }
            None => {
                let requests = sqlx::query_as::<_, Request>("SELECT * FROM requests WHERE agent_id = ?")
                    .bind(agent_id)
                    .fetch_all(self.db_conn.get_pool())
                    .await
                    .unwrap_or(vec![]);
                return requests;
            }
        }
    }

    async fn find(&self, id: u64) -> Result<Request, DbError> {
        let request = sqlx::query_as::<_, Request>("SELECT * FROM requests WHERE id = ?")
            .bind(id)
            .fetch_one(self.db_conn.get_pool())
            .await;
        match request {
            Ok(request) => Ok(request),
            Err(e) => Err(DbError::SomethingWentWrong(e.to_string()))
        }
    }
}
