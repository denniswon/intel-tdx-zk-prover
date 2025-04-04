use crate::entity::request::{Request, RequestStatus};
use chrono::{DateTime, Utc};
use ethereum_types::{Address, U256};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

fn validate_fee_amount(value: &U256) -> Result<(), ValidationError> {
    if value < &U256::from(0) {
        return Err(ValidationError::new(
            "fee_amount must be greater than or equal to 0",
        ));
    }
    Ok(())
}

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct RequestRegisterDto {
    pub agent_id: i32,
    pub from_address: Address,
    #[validate(length(min = 1, message = "Prompt cannot be empty"))]
    pub prompt: String,
    pub request_data: Option<Vec<u8>>,
    #[validate(custom(
        function = "validate_fee_amount",
        message = "fee_amount must be greater than or equal to 0"
    ))]
    pub fee_amount: U256,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct RequestReadDto {
    pub id: i32,
    pub agent_id: i32,
    pub from_address: Address,
    pub prompt: String,
    pub request_data: Option<Vec<u8>>,
    pub fee_amount: U256,
    pub created_at: DateTime<Utc>,
    pub request_status: RequestStatus,
}

impl RequestReadDto {
    pub fn from(request: Request) -> RequestReadDto {
        Self {
            id: request.id,
            agent_id: request.agent_id,
            from_address: request.from_address,
            prompt: request.prompt,
            request_data: request.request_data,
            fee_amount: request.fee_amount,
            request_status: request.request_status,
            created_at: request.created_at,
        }
    }
}

impl std::fmt::Debug for RequestReadDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequestReadDto")
            .field("id", &self.id)
            .field("agent_id", &self.agent_id)
            .field("from_address", &self.from_address)
            .field("prompt", &self.prompt)
            .field("request_data", &self.request_data)
            .field("fee_amount", &self.fee_amount)
            .field("request_status", &self.request_status)
            .field("created_at", &self.created_at)
            .finish()
    }
}

impl std::fmt::Debug for RequestRegisterDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequestRegisterDto")
            .field("agent_id", &self.agent_id)
            .field("from_address", &self.from_address)
            .field("prompt", &self.prompt)
            .field("request_data", &self.request_data)
            .field("fee_amount", &self.fee_amount)
            .finish()
    }
}
