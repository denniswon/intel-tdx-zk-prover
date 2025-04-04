use crate::entity::attestation::{Attestation, AttestationType, VerificationStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct AttestationDto {
    pub id: i32,
    pub request_id: i32,
    pub attestation_type: AttestationType,
    pub verification_status: VerificationStatus,
    pub attestation_data: Option<Vec<u8>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct AttestationRegisterDto {
    pub request_id: i32,
    pub attestation_type: AttestationType,
    pub attestation_data: Vec<u8>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct AttestationReadDto {
    pub id: i32,
    pub request_id: i32,
    pub attestation_type: AttestationType,
    pub verification_status: VerificationStatus,
    pub attestation_data: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

impl AttestationReadDto {
    pub fn from(attestation: Attestation) -> AttestationReadDto {
        Self {
            id: attestation.id,
            request_id: attestation.request_id,
            attestation_type: attestation.attestation_type,
            verification_status: attestation.verification_status,
            attestation_data: attestation.attestation_data,
            created_at: attestation.created_at,
        }
    }
}

impl std::fmt::Debug for AttestationReadDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AttestationReadDto")
            .field("id", &self.id)
            .field("request_id", &self.request_id)
            .field("attestation_type", &self.attestation_type)
            .field("verification_status", &self.verification_status)
            .field("attestation_data", &self.attestation_data)
            .field("created_at", &self.created_at)
            .finish()
    }
}

impl std::fmt::Debug for AttestationRegisterDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AttestationRegisterDto")
            .field("request_id", &self.request_id)
            .field("attestation_type", &self.attestation_type)
            .field("attestation_data", &self.attestation_data)
            .finish()
    }
}
