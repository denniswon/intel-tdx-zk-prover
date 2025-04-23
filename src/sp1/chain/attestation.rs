use alloy::{
    primitives::{Bytes, Uint},
    sol,
    sol_types::{SolInterface, SolValue},
};

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

pub fn generate_prove_calldata(output: &[u8], proof: &[u8]) -> Vec<u8> {
    IProve::IProveCalls::proveRequest(
        IProve::proveRequestCall {
            request: RequestConfig {
                // TODO: fill in with real values
                nonce: Uint::from(0),
                creator: "0x0000000000000000000000000000000000000000".parse().unwrap(),
                operator: "0x0000000000000000000000000000000000000000".parse().unwrap(),
                model: "0x0000000000000000000000000000000000000000000000000000000000000000".parse().unwrap(),
                fee: Uint::from(0),
                deadline: Uint::from(0),
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
