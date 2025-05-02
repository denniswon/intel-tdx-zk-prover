use alloy::{
    primitives::{Bytes, Uint}, sol, sol_types::{SolInterface, SolValue}
};

use crate::entity::{quote::ProofType, request::OnchainRequest};

sol! {
    interface IAttestation {
        function verifyAndAttestWithZKProof(
            bytes calldata output,
            uint8 zk_coprocessor_type,
            bytes calldata proof
        ) returns (bool success, bytes memory output);
    }

    interface IProve {

        #[derive(Debug)]
        enum ProofType {
            RAWQUOTE,
            RISC0ZKP,
            SP1ZKP
        }

        #[derive(Debug)]
        struct RequestConfig {
            uint256 nonce;
            address creator;
            address operator;
            bytes32 model;
            uint256 fee;
            uint256 deadline;
        }

        function proveRequest(
            RequestConfig calldata request,
            ProofType zk_coprocessor_type,
            bytes calldata proof
        ) returns (bool success, bytes memory output);
    }
}

pub fn generate_attestation_calldata(output: &[u8], proof_type: ProofType, proof: &[u8]) -> Vec<u8> {
    let proof_type = match proof_type {
        ProofType::Sp1 => IProve::ProofType::SP1ZKP,
        ProofType::Risc0 => IProve::ProofType::RISC0ZKP,
    };

    IAttestation::IAttestationCalls::verifyAndAttestWithZKProof(
        IAttestation::verifyAndAttestWithZKProofCall {
            output: Bytes::from(output.to_vec()),
            zk_coprocessor_type: proof_type.into(),
            proof: Bytes::from(proof.to_vec()),
        },
    )
    .abi_encode()
}

pub fn generate_prove_calldata(request: &OnchainRequest, proof_type: ProofType, output: &[u8], proof: &[u8]) -> Vec<u8> {
    tracing::info!("Generating proveRequest calldata");
    let request_config = IProve::RequestConfig {
        nonce: Uint::from(request.nonce),
        creator: request.creator_address.parse().unwrap(),
        operator: request.operator_address.parse().unwrap(),
        model: request.model_id.parse().unwrap(),
        fee: Uint::from(request.fee_wei),
        deadline: Uint::from(request.deadline.timestamp()),
    };
    let proof_type = match proof_type {
        ProofType::Sp1 => IProve::ProofType::SP1ZKP,
        ProofType::Risc0 => IProve::ProofType::RISC0ZKP,
    };

    tracing::info!("ProveRequest RequestConfig: {:#?}", request_config);
    tracing::info!("ProveRequest Output: {:#?}", hex::encode(output));
    tracing::info!("ProveRequest Proof: {:#?}", hex::encode(proof));
    tracing::info!("ProveRequest Proof Type: {:#?}", proof_type);

    let proof_bytes = Bytes::from(concat_with_length_prefix(output, proof));
    tracing::info!("ProveRequest Proof (Bytes): {:#?}", proof_bytes);

    let calldata = IProve::IProveCalls::proveRequest(
        IProve::proveRequestCall {
            request: request_config,
            zk_coprocessor_type: proof_type,
            proof: proof_bytes,
        },
    )
    .abi_encode();
    tracing::info!("ProveRequest calldata: {}", hex::encode(&calldata));
    calldata
}

pub fn concat_with_length_prefix(output: &[u8], proof: &[u8]) -> Vec<u8> {
    let output_len = output.len();
    assert!(output_len <= u16::MAX as usize, "concat_with_length_prefix: output too large");

    let mut result = Vec::with_capacity(2 + output.len() + proof.len());

    // Encode output length as 2-byte big-endian u16
    let len_prefix = (output_len as u16).to_be_bytes();
    result.extend_from_slice(&len_prefix);

    // Append output and proof
    result.extend_from_slice(output);
    result.extend_from_slice(proof);

    result
}

pub fn decode_attestation_ret_data(ret: Vec<u8>) -> (bool, Vec<u8>) {
    let (verified, output) = <(bool, Bytes)>::abi_decode_params(&ret, true).unwrap();
    (verified, output.to_vec())
}
