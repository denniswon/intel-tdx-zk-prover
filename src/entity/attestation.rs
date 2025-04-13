use chrono::NaiveDateTime;
use dcap_qvl::quote::EnclaveReport;
use dcap_rs::types::TcbStatus;
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DcapVerifiedOutput {
    pub quote_version: u16,
    pub tee_type: u32,
    pub tcb_status: TcbStatus,
    pub fmspc: [u8; 6],
    pub quote_body: DcapQuoteBody,
    pub advisory_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum DcapQuoteBody {
    SGXQuoteBody(EnclaveReport),
    TD10QuoteBody(DcapTD10ReportBody)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DcapTD10ReportBody {
    pub tee_tcb_svn: [u8; 16],
    pub mrseam: Vec<u8>,
    pub mrsignerseam: Vec<u8>,
    pub seam_attributes: u64,
    pub td_attributes: u64,
    pub xfam: u64,
    pub mrtd: Vec<u8>,
    pub mrconfigid: Vec<u8>,
    pub mrowner: Vec<u8>,
    pub mrownerconfig: Vec<u8>,
    pub rtmr0: Vec<u8>,
    pub rtmr1: Vec<u8>,
    pub rtmr2: Vec<u8>,
    pub rtmr3: Vec<u8>,
    pub report_data: Vec<u8>,
}
