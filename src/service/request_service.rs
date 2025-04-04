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

    pub async fn create_request(
        &self,
        payload: RequestRegisterDto,
    ) -> Result<RequestReadDto, ApiError> {
        let request = self.add_request(payload).await;

        return match request {
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
        };
    }

    async fn add_request(&self, payload: RequestRegisterDto) -> Result<Request, SqlxError> {
        let insert = sqlx::query_as!(
            Request,
            r#"
                INSERT INTO requests (agent_id, from_address, prompt, request_data, fee_amount)
                VALUES ($1, $2, $3, $4, $5)
            "#,
            payload.agent_id,
            payload.from_address,
            payload.prompt,
            payload.request_data,
            BigDecimal::from_ethers_u256(&payload.fee_amount, 18).unwrap()
        )
        .execute(self.db_conn.get_pool())
        .await?;

        let request = self.request_repo.find(insert.last_insert_id()).await?;
        return Ok(request);
    }
}
