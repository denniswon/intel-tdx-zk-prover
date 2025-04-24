use alloy::{
    primitives::{Bytes, Uint},
    sol,
    sol_types::{SolInterface, SolValue},
};

use crate::entity::onchain_request::OnchainRequest;

sol! {
    interface IAttestation {
        function verifyAndAttestWithZKProof(
            bytes calldata output,
            uint8 zk_coprocessor_type,
            bytes calldata proof
        ) returns (bool success, bytes memory output);
    }

    interface IProve {
        function proveRequest(
            RequestConfig calldata request,
            ProofType zk_coprocessor_type,
            bytes calldata proof
        ) returns (bool success, bytes memory output);
    }

    enum ProofType {
        RAWQUOTE,
        RISC0ZKP,
        SP1ZKP
    }

    struct RequestConfig {
        uint256 nonce;
        address creator;
        address operator;
        bytes32 model;
        uint256 fee;
        uint256 deadline;
    }
}

pub fn generate_attestation_calldata(output: &[u8], proof: &[u8]) -> Vec<u8> {
    IAttestation::IAttestationCalls::verifyAndAttestWithZKProof(
        IAttestation::verifyAndAttestWithZKProofCall {
            output: Bytes::from(output.to_vec()),
            zk_coprocessor_type: 2,
            proof: Bytes::from(proof.to_vec()),
        },
    )
    .abi_encode()
}

pub fn generate_prove_calldata(request: &OnchainRequest, output: &[u8], proof: &[u8]) -> Vec<u8> {
    IProve::IProveCalls::proveRequest(
        IProve::proveRequestCall {
            request: RequestConfig {
                nonce: Uint::from(request.nonce),
                creator: request.creator_address.parse().unwrap(),
                operator: request.operator_address.parse().unwrap(),
                model: request.model_id.parse().unwrap(),
                fee: Uint::from(request.fee_wei),
                deadline: Uint::from(request.deadline.timestamp()),
            },
            zk_coprocessor_type: ProofType::SP1ZKP,
            proof: Bytes::from(concat_with_length_prefix(output, proof)),
        },
    )
    .abi_encode()
}

fn concat_with_length_prefix(output: &[u8], proof: &[u8]) -> Vec<u8> {
    let output_len = output.len() as u32;
    let mut result = Vec::with_capacity(4 + output.len() + proof.len());

    // Prefix: write output.len() as 4-byte little endian
    result.extend(&output_len.to_le_bytes());

    // Append output and proof
    result.extend(output);
    result.extend(proof);

    result
}

pub fn decode_attestation_ret_data(ret: Vec<u8>) -> (bool, Vec<u8>) {
    let (verified, output) = <(bool, Bytes)>::abi_decode_params(&ret, true).unwrap();
    (verified, output.to_vec())
}
