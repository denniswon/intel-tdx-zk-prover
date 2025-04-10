use crate::entity::{
    evm::EvmAddress,
    request::{Request, RequestStatus},
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct RequestRegisterDto {
    pub agent_id: i32,
    pub from_address: EvmAddress,
    pub request_data: Option<Vec<u8>>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct RequestReadDto {
    pub id: i32,
    pub agent_id: i32,
    pub from_address: EvmAddress,
    pub request_data: Option<Vec<u8>>,
    pub created_at: NaiveDateTime,
    pub request_status: RequestStatus,
}

impl RequestReadDto {
    pub fn from(request: Request) -> RequestReadDto {
        Self {
            id: request.id,
            agent_id: request.agent_id,
            from_address: request.from_address,
            request_data: request.request_data,
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
            .field("request_data", &self.request_data)
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
            .field("request_data", &self.request_data)
            .finish()
    }
}
