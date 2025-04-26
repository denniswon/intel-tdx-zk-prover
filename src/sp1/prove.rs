#![allow(dead_code)]

use crate::entity::onchain_request::OnchainRequest;
use crate::entity::quote::{ProofType, TdxQuoteStatus};
use crate::chain::attestation::{decode_attestation_ret_data, generate_attestation_calldata, generate_prove_calldata};
use crate::chain::pccs::enclave_id::{get_enclave_identity, EnclaveIdType};
use crate::chain::pccs::fmspc_tcb::get_tcb_info;
use crate::chain::pccs::pcs::get_certificate_by_id;
use crate::chain::pccs::pcs::IPCSDao::CA;
use crate::chain::TxSender;
use crate::config::parameter;
use crate::chain::constants::{AUTOMATA_DEFAULT_DCAP_CONTRACT, AUTOMATA_DEFAULT_RPC_URL};
use crate::chain::pccs::parser::get_pck_fmspc_and_issuer;
use crate::risc0::code::DCAP_GUEST_ELF;

use alloy::primitives::TxHash;
use alloy_chains::NamedChain;
use anyhow::{anyhow, Result};
use dcap_rs::constants::{SGX_TEE_TYPE, TDX_TEE_TYPE};
use dcap_rs::types::{collaterals::IntelCollateral, VerifiedOutput};
use risc0_zkvm::{compute_image_id, default_prover, Digest, ExecutorEnv, ProverOpts};
use serde::{Deserialize, Serialize};
use sp1_sdk::{HashableKey, ProverClient, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey};
use validator::Validate;

pub const DCAP_ELF: &[u8] = include_bytes!("../../elf/dcap-sp1-guest-program-elf");

/// Enum representing the available proof systems
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ProofSystem {
    Groth16,
    Plonk,
}

/// Enum representing the available proof systems
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ZkvmProof {
    Sp1((SP1VerifyingKey, SP1ProofWithPublicValues)),
    Risc0((Digest, Vec<u8>)),
}

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct DcapProof {
    exec_output: Vec<u8>,
    proof_output: Vec<u8>,
    image_id: Digest,
    seal: Vec<u8>,
    vk: SP1VerifyingKey,
    proof: SP1ProofWithPublicValues,
}

impl std::fmt::Debug for DcapProof {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DcapProof {{ exec_output: {:?}, proof_output: {:?}, vk: {:?}, proof: {:?} }}",
            self.exec_output,
            self.proof_output,
            self.vk.bytes32(),
            self.proof
        )
    }
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

// proof_system: [Optional] The proof system to use. Default: Groth16
pub async fn prove(quote: Vec<u8>, proof_type: ProofType, proof_system: Option<ProofSystem>) -> Result<ProofResponse> {
    tracing::debug!("Begin fetching the necessary collaterals...");
    // Step 1: Determine quote version and TEE type
    let quote_version = u16::from_le_bytes([quote[0], quote[1]]);
    let tee_type = u32::from_le_bytes([quote[4], quote[5], quote[6], quote[7]]);

    tracing::debug!("Quote version: {}", quote_version);
    tracing::debug!("TEE Type: {}", tee_type);

    if !(3..=4).contains(&quote_version) {
        return Err(anyhow!("Unsupported quote version {}", quote_version));
    }

    if tee_type != SGX_TEE_TYPE && tee_type != TDX_TEE_TYPE {
        return Err(anyhow!("Unsupported tee type {}", tee_type));
    }

    tracing::debug!("Quote read successfully. Begin fetching collaterals from the on-chain PCCS");

    let (root_ca, root_ca_crl) = get_certificate_by_id(CA::ROOT).await?;
    if root_ca.is_empty() || root_ca_crl.is_empty() {
        return Err(anyhow!("Intel SGX Root CA is missing"));
    } else {
        tracing::debug!("Fetched Intel SGX RootCA and CRL");
    }

    let (fmspc, pck_type, pck_issuer) = get_pck_fmspc_and_issuer(&quote, quote_version, tee_type);

    let tcb_type: u8 = if tee_type == TDX_TEE_TYPE { 1 } else { 0 };
    let tcb_version: u32 = if quote_version < 4 { 2 } else { 3 };
    let tcb_info = get_tcb_info(tcb_type, fmspc.as_str(), tcb_version).await?;

    tracing::debug!("Fetched TCBInfo JSON for FMSPC: {}", fmspc);

    let qe_id_type: EnclaveIdType = if tee_type == TDX_TEE_TYPE {
        EnclaveIdType::TDQE
    } else {
        EnclaveIdType::QE
    };
    let qe_identity = get_enclave_identity(qe_id_type, quote_version as u32).await?;
    tracing::debug!("Fetched QEIdentity JSON");

    let (signing_ca, _) = get_certificate_by_id(CA::SIGNING).await?;
    if signing_ca.is_empty() {
        return Err(anyhow!("Intel TCB Signing CA is missing"));
    } else {
        tracing::debug!("Fetched Intel TCB Signing CA");
    }

    let (_, pck_crl) = get_certificate_by_id(pck_type).await?;
    if pck_crl.is_empty() {
        return Err(anyhow!("CRL for {} is missing", pck_issuer));
    } else {
        tracing::debug!("Fetched Intel PCK CRL for {}", pck_issuer);
    }

    let mut intel_collaterals = IntelCollateral::new();
    tracing::debug!("set_tcbinfo_bytes: {:?}", tcb_info);
    intel_collaterals.set_tcbinfo_bytes(&tcb_info);
    tracing::debug!("set_qeidentity_bytes: {:?}", qe_identity);
    intel_collaterals.set_qeidentity_bytes(&qe_identity);
    tracing::debug!("set_intel_root_ca_der: {:?}", root_ca);
    intel_collaterals.set_intel_root_ca_der(&root_ca);
    tracing::debug!("set_sgx_tcb_signing_der: {:?}", signing_ca);
    intel_collaterals.set_sgx_tcb_signing_der(&signing_ca);
    tracing::debug!("set_sgx_intel_root_ca_crl_der: {:?}", root_ca_crl);
    intel_collaterals.set_sgx_intel_root_ca_crl_der(&root_ca_crl);
    tracing::debug!("set_sgx_platform_crl_der: {:?}", pck_crl);
    intel_collaterals.set_sgx_platform_crl_der(&pck_crl);

    let intel_collaterals_bytes = intel_collaterals.to_bytes();

    // Step 3: Generate the input to upload to Proving Server
    let input = generate_input(&quote, &intel_collaterals_bytes);

    tracing::info!("All collaterals found! Begin uploading input to SP1 Proving Server...");


    match proof_type {
        ProofType::Sp1 => {
            tracing::info!("Using Sp1 proof type");

            let mut stdin = SP1Stdin::new();
            stdin.write_slice(&input);

            let client = ProverClient::from_env();

            // Execute the program first
            let (ret, report) = client.execute(DCAP_ELF, &stdin).run().unwrap();
            tracing::debug!(
                "executed program with {} cycles",
                report.total_instruction_count()
            );
            // println!("{:?}", report);

            // Generate the proof
            let (pk, vk) = client.setup(DCAP_ELF);
            tracing::debug!("ProofSystem: {:?}", proof_system);
            let proof = if let Some(proof_system) = proof_system {
                if proof_system == ProofSystem::Groth16 {
                    client.prove(&pk, &stdin).groth16().run().unwrap()
                } else {
                    client.prove(&pk, &stdin).plonk().run().unwrap()
                }
            } else {
                client.prove(&pk, &stdin).groth16().run().unwrap()
            };

            let ret_slice = ret.as_slice();
            let output_len = u16::from_be_bytes([ret_slice[0], ret_slice[1]]) as usize;
            let mut output = Vec::with_capacity(output_len);
            output.extend_from_slice(&ret_slice[2..2 + output_len]);

            tracing::debug!("Execution Output: {}", hex::encode(ret_slice));
            tracing::debug!(
                "Proof pub value: {}",
                hex::encode(proof.public_values.as_slice())
            );
            tracing::debug!("VK: {}", vk.bytes32().to_string().as_str());
            tracing::debug!("Proof: {}", hex::encode(proof.bytes()));

            let dcap_proof = DcapProof { exec_output: ret_slice.to_vec(), proof_output: output, vk, proof };

            Ok(ProofResponse { proof: dcap_proof, proof_type: ProofType::Sp1, prover_request_id: None })
        }
        ProofType::Risc0 => {
            tracing::info!("Using Risc0 proof type");

            // Step 3: Generate the input to upload to Bonsai
            let image_id = compute_image_id(DCAP_GUEST_ELF)?;
            log::info!("Image ID: {}", image_id.to_string());

            let input = generate_input(&quote, &serialized_collaterals);
            println!("All collaterals found! Begin uploading input to Bonsai...");

            // Set RISC0_PROVER env to bonsai
            std::env::set_var("RISC0_PROVER", "bonsai");

            let env = ExecutorEnv::builder().write_slice(&input).build()?;
            let receipt = default_prover()
                .prove_with_opts(env, DCAP_GUEST_ELF, &ProverOpts::groth16())?
                .receipt;
            receipt.verify(image_id)?;

            let output;
            let seal;
            if let Groth16(snark_receipt) = receipt.inner {
                output = receipt.journal.bytes;
                seal = groth16::encode(snark_receipt.seal)?;
            } else {
                return Err(anyhow!("Not a Groth16 Receipt"));
            }

            let mut offset: usize = 0;
            let output_len = u16::from_be_bytes(output[offset..offset + 2].try_into().unwrap());
            offset += 2;
            let raw_verified_output = &output[offset..offset + output_len as usize];
            let verified_output = VerifiedOutput::from_bytes(raw_verified_output);
            offset += output_len as usize;
            let current_time = u64::from_be_bytes(output[offset..offset + 8].try_into().unwrap());
            offset += 8;
            let tcbinfo_root_hash = &output[offset..offset + 32];
            offset += 32;
            let enclaveidentity_root_hash = &output[offset..offset + 32];
            offset += 32;
            let root_cert_hash = &output[offset..offset + 32];
            offset += 32;
            let signing_cert_hash = &output[offset..offset + 32];
            offset += 32;
            let root_crl_hash = &output[offset..offset + 32];
            offset += 32;
            let pck_crl_hash = &output[offset..offset + 32];

            println!("Verified Output: {:?}", verified_output);
            log::info!("Timestamp: {}", current_time);
            log::info!("TCB Info Root Hash: {}", hex::encode(&tcbinfo_root_hash));
            log::info!("Enclave Identity Root Hash: {}", hex::encode(&enclaveidentity_root_hash));
            log::info!("Root Cert Hash: {}", hex::encode(&root_cert_hash));
            log::info!("Signing Cert Hash: {}", hex::encode(&signing_cert_hash));
            log::info!("Root CRL hash: {}", hex::encode(&root_crl_hash));
            log::info!("PCK CRL hash: {}", hex::encode(&pck_crl_hash));

            println!("Journal: {}", hex::encode(&output));
            println!("seal: {}", hex::encode(&seal));

            let dcap_proof = DcapProof { exec_output: output, proof_output: output, image_id, seal };

            Ok(ProofResponse { proof: dcap_proof, proof_type: ProofType::Risc0, prover_request_id: None })
        }
    }
}

pub async fn verify_proof(proof: DcapProof) -> Result<VerifiedOutput> {
    // Verify proof
    let client = ProverClient::from_env();

    client
        .verify(&proof.proof, &proof.vk)
        .expect("Failed to verify proof");
    tracing::info!("Successfully verified proof.");

    let parsed_output = VerifiedOutput::from_bytes(&proof.proof_output);
    tracing::info!("{:?}", parsed_output);
    Ok(parsed_output)
}

pub async fn submit_proof(
    request: OnchainRequest,
    proof: DcapProof,
) -> Result<(bool, Vec<u8>, Option<TxHash>, Option<SubmitProofResponse>)> {
    // Send the calldata to Ethereum.
    tracing::info!("Submitting proofs to on-chain DCAP contract to be verified...");

    let verify_only = parameter::get("VERIFY_ONLY");

    match verify_only.as_str() {
        "true" => {
            tracing::info!("Verify only mode enabled");

            let tx_sender = TxSender::new(
                AUTOMATA_DEFAULT_RPC_URL,
                AUTOMATA_DEFAULT_DCAP_CONTRACT,
                None,
                Some(parameter::get("PROVER_PRIVATE_KEY").as_str())
            ).expect("Failed to create txSender");

            // staticcall to the Halo prove request contract to verify proof
            let calldata = generate_attestation_calldata(&proof.exec_output, &proof.proof.bytes());
            tracing::info!("Calldata: {}", hex::encode(&calldata));
            let call_output = (tx_sender.call(calldata.clone()).await?).to_vec();
            tracing::info!("Call output: {}", hex::encode(&call_output));
            let (chain_verified, chain_raw_verified_output) = decode_attestation_ret_data(call_output);
            tracing::info!("Chain verified: {}", chain_verified);
            tracing::info!("Chain raw verified output: {}", hex::encode(&chain_raw_verified_output));

            if chain_verified && proof.proof_output == chain_raw_verified_output {
                tracing::info!("On-chain verification succeed.");
            } else {
                tracing::error!("On-chain verification fail!");
            }
            Ok((chain_verified, chain_raw_verified_output, None, None))
        },
        _ => {
            tracing::info!("Submitting proof transaction...");

            let tx_sender = TxSender::new(
                parameter::get("DEFAULT_RPC_URL").as_str(),
                parameter::get("DEFAULT_DCAP_CONTRACT").as_str(),
                Some(NamedChain::Base),
                Some(parameter::get("PROVER_PRIVATE_KEY").as_str())
            ).expect("Failed to create txSender");

            let calldata = generate_prove_calldata(&request, &proof.exec_output, &proof.proof.bytes());
            tracing::info!("Calldata: {}", hex::encode(&calldata));
            // submit proof transaction to Halo contract to verify proof
            match tx_sender.send(calldata.clone()).await {
                Ok((tx_hash, receipt)) => {
                    tracing::info!("Transaction hash: {}", tx_hash);
                    tracing::info!("Transaction receipt: {:#?}", receipt);
                    match receipt {
                        Some(_receipt) => {
                            Ok((true, proof.proof_output, Some(tx_hash), Some(SubmitProofResponse {
                                transaction_hash: tx_hash,
                                proof_type: ProofType::Sp1,
                                status: TdxQuoteStatus::Success
                            })))
                        },
                        None => {
                            Ok((false, proof.proof_output, Some(tx_hash), None))
                        }
                    }
                },
                Err(e) => {
                    tracing::error!("Failed to submit proof transaction: {}", e);
                    Ok((false, proof.proof_output, None, None))
                }
            }
        }
    }
}

pub fn deserialize_output(proof: DcapProof) -> VerifiedOutput {
    let deserialized_output = VerifiedOutput::from_bytes(&proof.proof_output);
    tracing::info!("Deserialized output: {:?}", deserialized_output);
    deserialized_output
}

fn generate_input(quote: &[u8], collaterals: &[u8]) -> Vec<u8> {
    // get current time in seconds since epoch
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let current_time_bytes = current_time.to_le_bytes();

    let quote_len = quote.len() as u32;
    let intel_collaterals_bytes_len = collaterals.len() as u32;
    let total_len = 8 + 4 + 4 + quote_len + intel_collaterals_bytes_len;

    let mut input = Vec::with_capacity(total_len as usize);
    input.extend_from_slice(&current_time_bytes);
    input.extend_from_slice(&quote_len.to_le_bytes());
    input.extend_from_slice(&intel_collaterals_bytes_len.to_le_bytes());
    input.extend_from_slice(quote);
    input.extend_from_slice(collaterals);

    input.to_owned()
}
