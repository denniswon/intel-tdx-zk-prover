#![allow(dead_code)]

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
    Sp1((Vec<u8> /* journal */, SP1VerifyingKey /* vk */, SP1ProofWithPublicValues /* proof */)),
    Risc0((Receipt /* receipt: contains journal */, Digest /* image_id */, Vec<u8> /* seal */)),
}

impl std::fmt::Debug for ZkvmProof {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZkvmProof::Sp1((output, vk, proof)) =>
                write!(f, "Sp1 {{ proof_bytes: {:?}, vk: {:?}, proof: {:?} }}",
                    hex::encode(output), hex::encode(vk.bytes32()), proof),
            ZkvmProof::Risc0((receipt, image_id, seal)) =>
                write!(f, "Risc0 {{ journal: {:?}, image_id: {:?}, seal: {:?} }}",
                    hex::encode(receipt.journal.bytes.clone()), hex::encode(image_id), hex::encode(seal)),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct DcapProof {
    pub verified_output: Vec<u8>,
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
