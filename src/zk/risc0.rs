use anyhow::Result;
use risc0_ethereum_contracts::groth16;
use risc0_zkvm::{
    compute_image_id, default_prover, ExecutorEnv, InnerReceipt::Groth16, ProverOpts,
};
use tokio::task;
use crate::{
    entity::{quote::ProofType, zk::{DcapProof, ProofResponse, ProofSystem, ZkvmProof, DCAP_RISC0_ELF}},
    zk::extract_proof_output
};

use dcap_rs::types::VerifiedOutput;

// proof_system: [Optional] The proof system to use. Default: Groth16
pub async fn prove(collateral_input: Vec<u8>, proof_system: Option<ProofSystem>) -> Result<ProofResponse> {
    tracing::info!("Begin uploading input to Bonsai...");

    // Set RISC0_PROVER env to bonsai
    std::env::set_var("RISC0_PROVER", "bonsai");

    // BonsaiProver uses the reqwest blocking client as the default (and only option).
    // It will cause issues when running in async contexts unless explicitly ran in a task that can block (context)
    let receipt = task::spawn_blocking(move || {
        let env = ExecutorEnv::builder().write_slice(&collateral_input).build().unwrap();
        default_prover()
            .prove_with_opts(env, DCAP_RISC0_ELF, &ProverOpts::groth16()).unwrap()
            .receipt
    }).await?;

    let image_id = compute_image_id(DCAP_RISC0_ELF).unwrap();
    receipt.verify(image_id)?;

    let _receipt = receipt.clone();
    let journal;
    let seal;
    match _receipt.inner {
        Groth16(snark_receipt) => {
            journal = _receipt.journal.bytes.clone();
            seal = groth16::encode(snark_receipt.seal)?;
        },
        _ => {
            return Err(
                anyhow::anyhow!("Proof system {:#?} is not Groth16, which is not supported yet",
                proof_system.unwrap_or(ProofSystem::Groth16))
            );
        }
    }

    let mut offset: usize = 0;
    let raw_verified_output = extract_proof_output(journal.clone());
    let verified_output = VerifiedOutput::from_bytes(&raw_verified_output);
    offset += raw_verified_output.len();
    let current_time = u64::from_be_bytes(journal[offset..offset + 8].try_into().unwrap());
    offset += 8;
    let tcbinfo_root_hash = &journal[offset..offset + 32];
    offset += 32;
    let enclaveidentity_root_hash = &journal[offset..offset + 32];
    offset += 32;
    let root_cert_hash = &journal[offset..offset + 32];
    offset += 32;
    let signing_cert_hash = &journal[offset..offset + 32];
    offset += 32;
    let root_crl_hash = &journal[offset..offset + 32];
    offset += 32;
    let pck_crl_hash = &journal[offset..offset + 32];

    tracing::info!("Verified Output: {:?}", verified_output);
    tracing::info!("Timestamp: {}", current_time);
    tracing::info!("TCB Info Root Hash: {}", hex::encode(&tcbinfo_root_hash));
    tracing::info!("Enclave Identity Root Hash: {}", hex::encode(&enclaveidentity_root_hash));
    tracing::info!("Root Cert Hash: {}", hex::encode(&root_cert_hash));
    tracing::info!("Signing Cert Hash: {}", hex::encode(&signing_cert_hash));
    tracing::info!("Root CRL hash: {}", hex::encode(&root_crl_hash));
    tracing::info!("PCK CRL hash: {}", hex::encode(&pck_crl_hash));

    tracing::info!("Journal: {}", hex::encode(&journal.clone()));
    tracing::info!("Seal: {}", hex::encode(&seal));

    let dcap_proof = DcapProof {
        verified_output: raw_verified_output,
        proof: ZkvmProof::Risc0((receipt, image_id, seal)),
    };

    Ok(ProofResponse { proof: dcap_proof, proof_type: ProofType::Risc0, prover_request_id: None })
}
