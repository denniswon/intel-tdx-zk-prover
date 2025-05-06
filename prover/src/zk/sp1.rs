#![allow(dead_code)]

use std::time::Duration;

use crate::{entity::{
    quote::ProofType,
    zk::{DcapProof, ProofResponse, ProofSystem, ZkvmProof, DCAP_SP1_ELF}
}, zk::extract_proof_output};

use anyhow::Result;
use sp1_sdk::{network::FulfillmentStrategy, HashableKey, Prover, ProverClient, SP1Stdin};

// proof_system: [Optional] The proof system to use. Default: Groth16
pub async fn prove(collateral_input: Vec<u8>, proof_system: Option<ProofSystem>) -> Result<ProofResponse> {
    tracing::info!("Using Sp1 proof type");

    let mut stdin = SP1Stdin::new();
    stdin.write_slice(&collateral_input);

    let client = ProverClient::builder().network().build();

    if std::env::var("ENV").unwrap_or("dev".to_string()) != "prod" {
        // Execute the program first
        let (_journal, report) = client.execute(DCAP_SP1_ELF, &stdin).run().unwrap();
        tracing::debug!(
            "executed program with {} cycles",
            report.total_instruction_count()
        );
    }

    // Generate the proof
    let (pk, vk) = client.setup(DCAP_SP1_ELF);
    tracing::debug!("ProofSystem: {:?}", proof_system);
    let prover_request_id = if let Some(proof_system) = proof_system {
        if proof_system == ProofSystem::Groth16 {
            client.prove(&pk, &stdin)
                .groth16()
                .skip_simulation(true)
                .strategy(FulfillmentStrategy::Reserved)
                .request_async()
                .await
                .unwrap()
        } else {
            client.prove(&pk, &stdin)
                .plonk()
                .skip_simulation(true)
                .strategy(FulfillmentStrategy::Reserved)
                .request_async()
                .await
                .unwrap()
        }
    } else {
        client.prove(&pk, &stdin)
            .groth16()
            .skip_simulation(true)
            .strategy(FulfillmentStrategy::Reserved)
            .request_async()
            .await
            .unwrap()
    };
    tracing::info!("Prover Request ID: {}", hex::encode(prover_request_id));

    // Wait for proof complete with a timeout
    let proof = client.wait_proof(
        prover_request_id,
        Some(Duration::from_secs(15 * 60))
    ).await.unwrap();

    let journal = proof.public_values.as_slice();
    let raw_verified_output = extract_proof_output(journal.to_vec());

    tracing::debug!("Execution Output (journal): {}", hex::encode(journal));
    tracing::debug!("Proof pub value: {}", hex::encode(proof.public_values.as_slice()));
    tracing::debug!("VK: {}", vk.bytes32().to_string().as_str());
    tracing::debug!("Proof: {}", hex::encode(proof.bytes()));

    let zk_proof = ZkvmProof::Sp1((journal.to_vec(), vk, proof));
    let dcap_proof = DcapProof { verified_output: raw_verified_output, proof: zk_proof };

    Ok(ProofResponse {
        proof: dcap_proof,
        proof_type: ProofType::Sp1,
        prover_request_id: Some(prover_request_id.to_vec())
    })
}
