use crate::config::database::{Database, DatabaseTrait};
use crate::dto::request_dto::{RequestReadDto, RequestRegisterDto};
use crate::entity::request::Request;
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::repository::request_repository::{RequestRepository, RequestRepositoryTrait};
use sqlx::Error as SqlxError;
use std::sync::Arc;

#[derive(Clone)]
pub struct RequestService {
    request_repo: RequestRepository,
    db_conn: Arc<Database>,
}

impl RequestService {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            request_repo: RequestRepository::new(db_conn),
            db_conn: Arc::clone(db_conn),
        }
    }

    pub async fn create(&self, payload: RequestRegisterDto) -> Result<RequestReadDto, ApiError> {
        let request = self.add_request(payload).await;

        match request {
            Ok(request) => Ok(RequestReadDto::from(request)),
            Err(e) => match e {
                SqlxError::Database(e) => match e.code() {
                    Some(code) => {
                        if code == "23000" {
                            Err(DbError::UniqueConstraintViolation(e.to_string()))?
                        } else {
                            Err(DbError::SomethingWentWrong(e.to_string()))?
                        }
                    }
                    _ => Err(DbError::SomethingWentWrong(e.to_string()))?,
                },
                _ => Err(DbError::SomethingWentWrong(e.to_string()))?,
            },
        }
    }

    async fn add_request(&self, payload: RequestRegisterDto) -> Result<Request, SqlxError> {
        let request = sqlx::query_as!(
            Request,
            r#"
                INSERT INTO requests (agent_id, from_address, request_data)
                VALUES ($1, $2, $3)
                RETURNING
                id,
                agent_id,
                from_address,
                request_data,
                request_status as "request_status: _",
                created_at as "created_at: _"
            "#,
            payload.agent_id,
            payload.from_address.to_string(),
            payload.request_data.unwrap_or_default(),
        )
        .fetch_one(self.db_conn.get_pool())
        .await?;
        Ok(request)
    }

    pub async fn delete(&self, id: i32) -> Result<RequestReadDto, ApiError> {
        let request = self.delete_request(id).await;
        match request {
            Ok(request) => Ok(RequestReadDto::from(request)),
            Err(e) => Err(DbError::SomethingWentWrong(e.to_string()))?,
        }
    }

    pub async fn delete_request(&self, id: i32) -> Result<Request, SqlxError> {
        let request = sqlx::query_as!(
            Request,
            r#"
                DELETE FROM requests
                WHERE id = $1
                RETURNING
                id,
                agent_id,
                from_address,
                request_data,
                request_status as "request_status: _",
                created_at as "created_at: _"
            "#,
            id,
        )
        .fetch_one(self.db_conn.get_pool())
        .await?;
        Ok(request)
    }
}
