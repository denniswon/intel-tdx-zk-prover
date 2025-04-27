use crate::config::database::{Database, DatabaseTrait};
use crate::dto::quote_dto::QuoteRegisterDto;
use crate::entity::quote::{ProofType, QuoteType, TdxQuote, TdxQuoteStatus};
use crate::entity::zk::DcapProof;
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::error::quote_error::QuoteError;
use crate::repository::quote_repository::{QuoteRepository, QuoteRepositoryTrait};
use crate::zk::{prove, verify_proof};

use dcap_rs::types::quotes::version_4::QuoteV4;
use dcap_rs::types::VerifiedOutput;
use sqlx::types::Uuid;
use sqlx::Error as SqlxError;
use std::sync::Arc;

use dcap_rs::types::quotes::version_3::QuoteV3;
use dcap_rs::types::collaterals::IntelCollateral;

use dcap_rs::utils::quotes::{
    version_3::verify_quote_dcapv3, 
    version_4::verify_quote_dcapv4
};

#[derive(Clone)]
pub struct QuoteService {
    quote_repo: QuoteRepository,
    db_conn: Arc<Database>,
}

impl QuoteService {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            quote_repo: QuoteRepository::new(db_conn),
            db_conn: Arc::clone(db_conn),
        }
    }

    pub async fn create_quote(&self, payload: QuoteRegisterDto) -> Result<TdxQuote, ApiError> {
        let quote = self.add_quote(payload).await;

        match quote {
            Ok(quote) => Ok(quote),
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

    async fn add_quote(&self, payload: QuoteRegisterDto) -> Result<TdxQuote, SqlxError> {
        let onchain_request_id = Uuid::parse_str(&payload.onchain_request_id).unwrap();
        let quote = sqlx::query_as!(
            TdxQuote,
            r#"
                INSERT INTO tdx_quote (onchain_request_id, status, quote)
                VALUES ($1, $2, decode($3, 'hex'))
                RETURNING
                id,
                onchain_request_id,
                status as "status: crate::entity::quote::TdxQuoteStatus",
                quote,
                proof_type as "proof_type: crate::entity::quote::ProofType",
                txn_hash,
                created_at as "created_at: _",
                updated_at as "updated_at: _",
                request_id as "request_id: _"
            "#,
            onchain_request_id,
            payload.status as TdxQuoteStatus,
            String::from_utf8(payload.quote.to_vec()).unwrap(),
        )
        .fetch_one(self.db_conn.get_pool())
        .await?;
        Ok(quote)
    }

    // Verify using onchain pccs collateral
    pub fn verify_dcap(&self, quote: TdxQuote, quote_type: Option<QuoteType>) -> Result<VerifiedOutput, QuoteError> {
        let quote = quote.quote;
        let quote_type = if let Some(quote_type) = quote_type {
            quote_type
        } else {
            QuoteType::DcapV4
        };

        let collateral = self.get_collateral(quote_type)?;
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

        match quote_type {
            QuoteType::DcapV3 => {
                let dcap_quote = QuoteV3::from_bytes(&quote);
                let verified_output = verify_quote_dcapv3(&dcap_quote, &collateral, now);
                Ok(verified_output)
            }
            QuoteType::DcapV4 => {
                let dcap_quote = QuoteV4::from_bytes(&quote);
                let verified_output = verify_quote_dcapv4(&dcap_quote, &collateral, now);
                Ok(verified_output)
            }
        }
    }

    pub fn get_collateral(&self, quote_type: QuoteType) -> Result<IntelCollateral, QuoteError> {
        let mut collaterals = IntelCollateral::new();
        match quote_type {
            QuoteType::DcapV3 => {
                collaterals.set_tcbinfo_bytes(include_bytes!("../../data/tcbinfov2.json"));
                collaterals.set_qeidentity_bytes(include_bytes!("../../data/qeidentityv2.json"));
                collaterals.set_intel_root_ca_der(include_bytes!("../../data/Intel_SGX_Provisioning_Certification_RootCA.cer"));
                collaterals.set_sgx_tcb_signing_pem(include_bytes!("../../data/signing_cert.pem"));
                collaterals.set_sgx_intel_root_ca_crl_der(include_bytes!("../../data/intel_root_ca_crl.der"));
                collaterals.set_sgx_platform_crl_der(include_bytes!("../../data/pck_platform_crl.der"));
                // collaterals.set_sgx_processor_crl_der(include_bytes!("../data/pck_processor_crl.der"));
            }
            QuoteType::DcapV4 => {
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

    pub async fn prove(&self, id: Uuid, proof_type: ProofType) -> Result<DcapProof, QuoteError> {
        let quote = self.quote_repo.find(id).await;

        match quote {
            Ok(quote) => {
                let proof = prove(quote.quote, proof_type, None).await;
                match proof {
                    Ok(proof) => Ok(proof.proof),
                    _ => Err(QuoteError::Invalid),
                }
            },
            _ => Err(QuoteError::Invalid),
        }
    }

    pub async fn verify(&self, proof: &DcapProof) -> Result<VerifiedOutput, QuoteError> {
        let result = verify_proof(proof).await;
        match result {
            Ok(output) => Ok(output),
            _ => Err(QuoteError::Invalid),
        }
    }

    pub async fn submit_proof(&self, proof: &DcapProof) -> Result<VerifiedOutput, QuoteError> {
        let result = verify_proof(proof).await;
        match result {
            Ok(output) => Ok(output),
            _ => Err(QuoteError::Invalid),
        }
    }
}
