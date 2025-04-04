use crate::config::database::{Database, DatabaseTrait};
use crate::dto::model_dto::{ModelReadDto, ModelRegisterDto};
use crate::entity::model::Model;
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::error::model_error::ModelError;
use crate::repository::model_repository::{ModelRepository, ModelRepositoryTrait};
use sqlx::Error as SqlxError;
use std::sync::Arc;

#[derive(Clone)]
pub struct ModelService {
    model_repo: ModelRepository,
    db_conn: Arc<Database>,
}

impl ModelService {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            model_repo: ModelRepository::new(db_conn),
            db_conn: Arc::clone(db_conn),
        }
    }

    pub async fn create_model(&self, payload: ModelRegisterDto) -> Result<ModelReadDto, ApiError> {
        let model = self.add_user(payload).await;

        return match user {
            Ok(user) => Ok(ModelReadDto::from(user)),
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

    async fn add_model(&self, payload: ModelRegisterDto) -> Result<Model, SqlxError> {
        let insert = sqlx::query_as!(
            Model,
            r#"
                INSERT INTO models (model_creator_address, model_name, model_description, model_type, model_uri)
                VALUES ($1, $2, $3, $4, $5)
            "#,
            payload.model_creator_address,
            payload.model_name,
            payload.model_description,
            payload.model_type,
            payload.model_uri
        )
        .execute(self.db_conn.get_pool())
        .await?;

        let model = self.model_repo.find(insert.last_insert_id()).await?;
        return Ok(model);
    }
}
