use alloy::primitives::TxHash;
use risc0_zkvm::{Digest, Receipt};
use serde::{Deserialize, Serialize};
use sp1_sdk::{HashableKey, SP1ProofWithPublicValues, SP1VerifyingKey};
use validator::Validate;

use super::quote::{ProofType, TdxQuoteStatus};

/// DCAP ELF for Sp1
pub const DCAP_SP1_ELF: &[u8] = include_bytes!("../../elf/dcap-sp1");
/// DCAP ELF for Risc0
pub const DCAP_RISC0_ELF: &[u8] = include_bytes!("../../elf/dcap-risc0");

/// Enum representing the available proof systems
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ProofSystem {
    Groth16,
    Plonk,
}

/// Enum representing the available proof systems
#[derive(Clone, Serialize, Deserialize)]
pub enum ZkvmProof {
    Sp1((Vec<u8>, SP1VerifyingKey, SP1ProofWithPublicValues)),
    Risc0((Receipt, Digest, Vec<u8>)),
}

impl std::fmt::Debug for ZkvmProof {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZkvmProof::Sp1((output, vk, proof)) =>
                write!(f, "Sp1 {{ output: {:?}, vk: {:?}, proof: {:?} }}", hex::encode(output), hex::encode(vk.bytes32()), proof),
            ZkvmProof::Risc0((receipt, digest, proof)) =>
                write!(f, "Risc0 {{ receipt: {:?}, digest: {:?}, proof: {:?} }}", receipt, hex::encode(digest), proof),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct DcapProof {
    pub output: Vec<u8>,
    pub proof: ZkvmProof,
}

#[derive(Clone, Validate)]
pub struct ProofResponse {
    pub proof: DcapProof,
    pub proof_type: ProofType,
    pub prover_request_id: Option<Vec<u8>>,
}

#[derive(Clone, Validate, Debug)]
pub struct SubmitProofResponse {
    pub transaction_hash: TxHash,
    pub proof_type: ProofType,
    pub status: TdxQuoteStatus,
}
