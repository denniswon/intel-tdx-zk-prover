use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ethereum_types::Address;

#[derive(Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct Model {
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
