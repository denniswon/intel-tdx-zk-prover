use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Attestation {
    pub id: i32,
    pub request_id: i32,
    pub attestation_type: AttestationType,
    pub verification_status: VerificationStatus,
    pub attestation_data: Vec<u8>,
    pub created_at: NaiveDateTime,
}


#[derive(strum_macros::Display, Debug, Clone, sqlx::Type, Deserialize, Serialize)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "verification_status", rename_all = "snake_case")]
pub enum VerificationStatus {
    Verified,
    Pending,
    Failed,
}

#[derive(strum_macros::Display, Debug, Clone, Copy, sqlx::Type, Deserialize, Serialize)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "attestation_type", rename_all = "snake_case")]
pub enum AttestationType {
    DcapV3,
    DcapV4,
}
