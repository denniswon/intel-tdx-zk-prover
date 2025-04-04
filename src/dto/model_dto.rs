use crate::entity::request::Request;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};
use ethereum_types::{U256, Address};

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct ModelDto {
    pub id: i32,
    pub model_creator_address: Address,
    pub model_name: String,
    pub model_description: String,
    pub model_type: String,
    pub model_uri: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_active: i8,
}

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct ModelRegisterDto {
    pub model_creator_address: Address,
    pub model_name: String,
    pub model_description: String,
    pub model_type: String,
    pub model_uri: String,
}

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct ModelRegisterDto {
    pub model_creator_address: Address,
    #[validate(length(min = 1, message = "Model name cannot be empty"))]
    pub model_name: String,
    pub model_description: String,
    #[validate(length(min = 1, message = "Model type cannot be empty"))]
    pub model_type: String,
    #[validate(length(min = 1, message = "Model URI cannot be empty"))]
    pub model_uri: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ModelReadDto {
    pub id: i32,
    pub model_creator_address: Address,
    pub model_name: String,
    pub model_description: String,
    pub model_type: String,
    pub model_uri: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_active: i8,
}

impl ModelReadDto {
    pub fn from(model: Model) -> ModelReadDto {
        Self {
            id: model.id,
            model_creator_address: model.model_creator_address,
            model_name: model.model_name,
            model_description: model.model_description,
            model_type: model.model_type,
            model_uri: model.model_uri,
            created_at: model.created_at,
            updated_at: model.updated_at,
            is_active: model.is_active,
        }
    }
}

impl std::fmt::Debug for ModelDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Model")
            .field("id", &self.id)
            .field("model_creator_address", &self.model_creator_address)
            .field("model_name", &self.model_name)
            .field("model_description", &self.model_description)
            .field("model_type", &self.model_type)
            .field("model_uri", &self.model_uri)
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .field("is_active", &self.is_active)
            .finish()
    }
}

impl std::fmt::Debug for ModelRegisterDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelRegisterDto")
            .field("model_creator_address", &self.model_creator_address)
            .field("model_name", &self.model_name)
            .field("model_description", &self.model_description)
            .field("model_type", &self.model_type)
            .field("model_uri", &self.model_uri)
            .finish()
    }
}
