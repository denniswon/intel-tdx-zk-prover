use crate::config::database::{Database, DatabaseTrait};
use crate::dto::attestation_dto::{AttestationReadDto, AttestationRegisterDto};
use crate::entity::attestation::{Attestation, AttestationType};
use crate::error::api_error::ApiError;
use crate::error::attestation_error::AttestationError;
use crate::error::db_error::DbError;
use crate::repository::attestation_repository::{AttestationRepository, AttestationRepositoryTrait};
use crate::sp1::prove::{prove, verify_proof, DcapProof};

use dcap_rs::types::quotes::version_4::QuoteV4;
use dcap_rs::types::VerifiedOutput;
use sqlx::Error as SqlxError;
use std::sync::Arc;

use dcap_rs::types::quotes::version_3::QuoteV3;
use dcap_rs::types::collaterals::IntelCollateral;

use dcap_rs::utils::quotes::{
    version_3::verify_quote_dcapv3, 
    version_4::verify_quote_dcapv4
};

#[derive(Clone)]
pub struct AttestationService {
    attestation_repo: AttestationRepository,
    db_conn: Arc<Database>,
}

impl AttestationService {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            attestation_repo: AttestationRepository::new(db_conn),
            db_conn: Arc::clone(db_conn),
        }
    }

    pub async fn create_attestation(&self, payload: AttestationRegisterDto) -> Result<AttestationReadDto, ApiError> {
        let attestation = self.add_attestation(payload).await;

        match attestation {
            Ok(attestation) => Ok(AttestationReadDto::from(attestation)),
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

    async fn add_attestation(&self, payload: AttestationRegisterDto) -> Result<Attestation, SqlxError> {
        let attestation = sqlx::query_as!(
            Attestation,
            r#"
                INSERT INTO attestations (request_id, attestation_type, attestation_data)
                VALUES ($1, $2, decode($3, 'hex'))
                RETURNING
                id,
                request_id,
                attestation_type as "attestation_type: _",
                verification_status as "verification_status: _",
                attestation_data,
                created_at as "created_at: _"
            "#,
            payload.request_id,
            payload.attestation_type as AttestationType,
            String::from_utf8(payload.attestation_data.to_vec()).unwrap(),
        )
        .fetch_one(self.db_conn.get_pool())
        .await?;
        Ok(attestation)
    }

    // Verify using onchain pccs collateral
    pub fn verify_dcap(&self, attestation: Attestation) -> Result<VerifiedOutput, AttestationError> {
        let quote = attestation.attestation_data;
        let collateral = self.get_collateral(attestation.attestation_type)?;
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

        match attestation.attestation_type {
            AttestationType::DcapV3 => {
                let dcap_quote = QuoteV3::from_bytes(&quote);
                let verified_output = verify_quote_dcapv3(&dcap_quote, &collateral, now);
                Ok(verified_output)
            }
            AttestationType::DcapV4 => {
                let dcap_quote = QuoteV4::from_bytes(&quote);
                let verified_output = verify_quote_dcapv4(&dcap_quote, &collateral, now);
                Ok(verified_output)
            }
        }
    }

    pub fn get_collateral(&self, attestation_type: AttestationType) -> Result<IntelCollateral, AttestationError> {
        let mut collaterals = IntelCollateral::new();
        match attestation_type {
            AttestationType::DcapV3 => {
                collaterals.set_tcbinfo_bytes(include_bytes!("../../data/tcbinfov2.json"));
                collaterals.set_qeidentity_bytes(include_bytes!("../../data/qeidentityv2.json"));
                collaterals.set_intel_root_ca_der(include_bytes!("../../data/Intel_SGX_Provisioning_Certification_RootCA.cer"));
                collaterals.set_sgx_tcb_signing_pem(include_bytes!("../../data/signing_cert.pem"));
                collaterals.set_sgx_intel_root_ca_crl_der(include_bytes!("../../data/intel_root_ca_crl.der"));
                collaterals.set_sgx_platform_crl_der(include_bytes!("../../data/pck_platform_crl.der"));
                // collaterals.set_sgx_processor_crl_der(include_bytes!("../data/pck_processor_crl.der"));
            }
            AttestationType::DcapV4 => {
                let mut collaterals = IntelCollateral::new();
                collaterals.set_tcbinfo_bytes(include_bytes!("../../data/tcbinfov3_00806f050000.json"));
                collaterals.set_qeidentity_bytes(include_bytes!("../../data/qeidentityv2_apiv4.json"));
                collaterals.set_intel_root_ca_der(include_bytes!("../../data/Intel_SGX_Provisioning_Certification_RootCA.cer"));
                collaterals.set_sgx_tcb_signing_pem(include_bytes!("../../data/signing_cert.pem"));
                collaterals.set_sgx_intel_root_ca_crl_der(include_bytes!("../../data/intel_root_ca_crl.der"));
                collaterals.set_sgx_platform_crl_der(include_bytes!("../../data/pck_platform_crl.der"));
                // collaterals.set_sgx_processor_crl_der(include_bytes!("../data/pck_processor_crl.der"));
            }
        }
        Ok(collaterals)
    }

    pub async fn prove(&self, id: i32) -> Result<DcapProof, AttestationError> {
        let attestation = self.attestation_repo.find(id.try_into().unwrap()).await;

        match attestation {
            Ok(attestation) => {
                let proof = prove(attestation.attestation_data, None).await;
                match proof {
                    Ok(proof) => Ok(proof),
                    _ => Err(AttestationError::Invalid),
                }
            },
            _ => Err(AttestationError::Invalid),
        }
    }

    pub async fn verify(&self, proof: DcapProof) -> Result<VerifiedOutput, AttestationError> {
        let result = verify_proof(proof).await;
        match result {
            Ok(output) => Ok(output),
            _ => Err(AttestationError::Invalid),
        }
    }

    pub async fn submit_proof(&self, proof: DcapProof) -> Result<VerifiedOutput, AttestationError> {
        let result = verify_proof(proof).await;
        match result {
            Ok(output) => Ok(output),
            _ => Err(AttestationError::Invalid),
        }
    }
}
