#![allow(dead_code)]

use crate::{entity::{
    quote::ProofType,
    zk::{DcapProof, ProofResponse, ProofSystem, ZkvmProof, DCAP_SP1_ELF}
}, zk::extract_proof_output};

use anyhow::Result;
use sp1_sdk::{HashableKey, ProverClient, SP1Stdin};

// proof_system: [Optional] The proof system to use. Default: Groth16
pub async fn prove(collateral_input: Vec<u8>, proof_system: Option<ProofSystem>) -> Result<ProofResponse> {
    tracing::info!("Using Sp1 proof type");

    let mut stdin = SP1Stdin::new();
    stdin.write_slice(&collateral_input);

    let client = ProverClient::from_env();

    // Execute the program first
    let (ret, report) = client.execute(DCAP_SP1_ELF, &stdin).run().unwrap();
    tracing::debug!(
        "executed program with {} cycles",
        report.total_instruction_count()
    );

    // Generate the proof
    let (pk, vk) = client.setup(DCAP_SP1_ELF);
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
    let output = extract_proof_output(ret_slice.to_vec());

    tracing::debug!("Execution Output: {}", hex::encode(ret_slice));
    tracing::debug!("Proof pub value: {}", hex::encode(proof.public_values.as_slice()));
    tracing::debug!("VK: {}", vk.bytes32().to_string().as_str());
    tracing::debug!("Proof: {}", hex::encode(proof.bytes()));

    let zkvm_proof = ZkvmProof::Sp1((output, vk, proof));
    let dcap_proof = DcapProof { output: ret_slice.to_vec(), proof: zkvm_proof };

    Ok(ProofResponse { proof: dcap_proof, proof_type: ProofType::Sp1, prover_request_id: None })
}
