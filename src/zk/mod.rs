#![allow(dead_code)]

pub mod risc0;
pub mod sp1;

use crate::entity::request::OnchainRequest;
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
use crate::entity::zk::{DcapProof, ProofResponse, ProofSystem, SubmitProofResponse, ZkvmProof};
use crate::zk::sp1::prove as sp1_prove;
use crate::zk::risc0::prove as risc0_prove;

use alloy::primitives::TxHash;
use alloy_chains::NamedChain;
use anyhow::{anyhow, Result};
use dcap_rs::constants::{SGX_TEE_TYPE, TDX_TEE_TYPE};
use dcap_rs::types::{collaterals::IntelCollateral, VerifiedOutput};
use sp1_sdk::ProverClient;

// proof_system: [Optional] The proof system to use. Default: Groth16
pub async fn prove(quote: Vec<u8>, proof_type: ProofType, proof_system: Option<ProofSystem>) -> Result<ProofResponse> {
    tracing::info!("Begin fetching the necessary collaterals...");
    // Step 1: Determine quote version and TEE type
    let quote_version = u16::from_le_bytes([quote[0], quote[1]]);
    let tee_type = u32::from_le_bytes([quote[4], quote[5], quote[6], quote[7]]);

    tracing::info!("Quote version: {}", quote_version);
    tracing::info!("TEE Type: {}", tee_type);

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

    tracing::info!("All collaterals found! Begin uploading input to Proving Server...");

    match proof_type {
        ProofType::Sp1 => sp1_prove(input, proof_system).await,
        ProofType::Risc0 => risc0_prove(input, proof_system).await
    }
}

pub async fn verify_proof(proof: DcapProof) -> Result<VerifiedOutput> {
    match proof.proof {
        ZkvmProof::Sp1((_output, vk, proof)) => {
            ProverClient::from_env().verify(&proof, &vk)?;
        },
        ZkvmProof::Risc0((receipt, image_id, _proof)) => {
            receipt.verify(image_id)?;
        }
    }
    let verified_output = VerifiedOutput::from_bytes(&proof.output);
    Ok(verified_output)
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
            let calldata = generate_attestation_calldata(&proof.output, &proof.proof.bytes());
            tracing::info!("Calldata: {}", hex::encode(&calldata));
            let call_output = (tx_sender.call(calldata.clone()).await?).to_vec();
            tracing::info!("Call output: {}", hex::encode(&call_output));
            let (chain_verified, chain_raw_verified_output) = decode_attestation_ret_data(call_output);
            tracing::info!("Chain verified: {}", chain_verified);
            tracing::info!("Chain raw verified output: {}", hex::encode(&chain_raw_verified_output));

            if chain_verified {
                match proof.proof {
                    ZkvmProof::Sp1((output, vk, proof)) => {
                        if output != chain_raw_verified_output {
                            tracing::info!("On-chain verification succeed.");
                            return Ok((true, chain_raw_verified_output, None, None));
                        }
                    },
                    ZkvmProof::Risc0((receipt, image_id, seal)) => {
                        let mut offset: usize = 0;
                        let output_len = u16::from_be_bytes(proof.output[offset..offset + 2].try_into().unwrap());
                        offset += 2;
                        let raw_verified_output = &proof.output[offset..offset + output_len as usize];

                        if proof.output != chain_raw_verified_output {
                            tracing::info!("On-chain verification succeed.");
                            return Ok((true, chain_raw_verified_output, None, None));
                        }
                    }
                }
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

            let calldata = match proof.proof {
                ZkvmProof::Sp1((output, vk, zk_proof)) => {
                    generate_prove_calldata(&request, &proof.output, &zk_proof.bytes())
                },
                ZkvmProof::Risc0((receipt, digest, zk_proof)) => {
                    generate_prove_calldata(&request, &proof.output, &zk_proof)
                }
            };
            tracing::info!("Calldata: {}", hex::encode(&calldata));
            // submit proof transaction to Halo contract to verify proof
            match tx_sender.send(calldata.clone()).await {
                Ok((tx_hash, receipt)) => {
                    tracing::info!("Transaction hash: {}", tx_hash);
                    tracing::info!("Transaction receipt: {:#?}", receipt);
                    match receipt {
                        Some(_receipt) => {
                            Ok((true, proof.output, Some(tx_hash), Some(SubmitProofResponse {
                                transaction_hash: tx_hash,
                                proof_type: ProofType::Sp1,
                                status: TdxQuoteStatus::Success
                            })))
                        },
                        None => {
                            Ok((false, proof.output, Some(tx_hash), None))
                        }
                    }
                },
                Err(e) => {
                    tracing::error!("Failed to submit proof transaction: {}", e);
                    Ok((false, proof.output, None, None))
                }
            }
        }
    }
}

pub fn extract_proof_output(execution_output: Vec<u8>) -> Vec<u8> {
    let output_len = u16::from_be_bytes([execution_output[0], execution_output[1]]) as usize;
    let mut output = Vec::with_capacity(output_len);
    output.extend_from_slice(&execution_output[2..2 + output_len]);
    output
}

pub fn deserialize_output(proof: DcapProof) -> VerifiedOutput {
    let proof_output = extract_proof_output(proof.output);
    let deserialized_output = VerifiedOutput::from_bytes(&proof_output);
    tracing::debug!("Deserialized output: {:?}", deserialized_output);
    deserialized_output
}

pub fn generate_input(quote: &[u8], collaterals: &[u8]) -> Vec<u8> {
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

#[cfg(test)]
mod tests {
    use alloy::{primitives::{Bytes, Uint}, sol_types::SolInterface};
    use alloy_chains::NamedChain;
    use x509_parser::nom::AsBytes;

    use crate::chain::{
        attestation::{concat_with_length_prefix, decode_attestation_ret_data, IProve::{self, RequestConfig}},
        TxSender
    };

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn submit_proof() {
        let request_config = RequestConfig {
            nonce: Uint::from(94),
            creator: "0x6BBC359046BDBFb1596222E6257F0ef24e0Fc0B9".parse().unwrap(),
            operator: "0xEeE7FB850D28f5cabd5f1EDF540646b5bEA17CE5".parse().unwrap(),
            model: "0x682db2fe997945208caa888543ffca2ad2c7edf1ab0b02899b9977e6d18af477".parse().unwrap(),
            fee: Uint::from(0),
            deadline: Uint::from(1745791225),
        };
        tracing::info!("ProveRequest RequestConfig: {:#?}", request_config);

        let output = hex::decode("02550004810000000020a06f00000007010300000000000000000000000000c51e5cb16c461fe29b60394984755325ecd05a9a7a8fb3a116f1c3cf0aca4b0eb9edefb9b404deeaee4b7d454372d17a000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000702000000000000c68518a0ebb42136c12b2275164f8c72f25fa9a34392228687ed6e9caeb9c0f1dbd895e9cf475121c029dc47e70e91fd00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000085e0855a6384fa1c8a6ab36d0dcbfaa11a5753e5a070c08218ae5fe872fcb86967fd2449c29e22e59dc9fec998cb65474a7db64a609c77e85f603c23e9a9fd03bfd9e6b52ce527f774a598e66d58386026cea79b2aea13b81a0b70cfacdec0ca8a4fe048fea22663152ef128853caa5c033cbe66baf32ba1ff7f6b1afc1624c279f50a4cbc522a735ca6f69551e61ef2efb98b5bae8f04d99c50a0174182dca782a2a6f5b891f5a09bd5887bb8904cb0814f6c3d00026953a5c45d8abfd22c8d0000000000000000000000000000000000000000000000000000000000000000a7654f588b28ef3b4833e50f8c2b001d4c67a8164e86b4d1fbf4db149c1f5ac200000000680c04570b2e5424728531e3183fa52906f9ff882ddff3cbccd3b19e2c418bbae9ccf30aa7fa01fc7a25a72b367cd8bd6aed0bb37108920a3292f557465b91fac3a68eb10fa74a3f32c80b978c8ad671395dabf24283eef9091bc3919fd39b9915a87f1adf3061c165c0191e2658256a2855cac9267f179aafb1990c9e918d6452816adf88b0758c525b2f28ee1896907de49511ffb1d919b04bd65b91943be6ebb0a5fe8442f7d95513d4e780cc15c5cd72d9395828137c632877fded3ba0ad43efd4e9").unwrap();
        let proof = hex::decode("11b6a09d074d81b271dd2256c6cbb1be23b67264c4e871cf0d81ca83e676ec03fcb5f1db2a09a910b27807ac4998523f2bf7580dd1fe5e84742bddea572cd1570fcdb95801ecd7af28ebd7afc4c4adb89645807a95b2b68710507828a6c2fcf767cdc8be239e2e278a00ea7dd08a6a346bd34f5865567961244bb3960a6d0322b683ec070f260184519f5c55fb540407c1ec046b86642e0f1300ca2bd7d52b9a56ab4fcf2f20e7f72338d012e7a3e8201a3b87a1dc91efea21ef25705f642ced9652915b075e29c4cf14e408eb9e682ea928a2f5373e39d3d129272ae6d78ee8da8846240a67a5cd373919d0c0b889d4814e21cb9376ee60e59996d23fb5599b3a6295ff").unwrap();
        let proof_bytes = concat_with_length_prefix(output.as_bytes(), proof.as_bytes());
        tracing::info!("ProveRequest Proof (Bytes): {:#?}", Bytes::from(proof_bytes.clone()));

        let tx_sender = TxSender::new(
            "https://sepolia.base.org",
            "0x9E4a45c40e06CE0653C33769138dF48802c1CF1e",
            Some(NamedChain::BaseSepolia),
            None
        ).expect("Failed to create txSender");

        let calldata = IProve::IProveCalls::proveRequest(
            IProve::proveRequestCall {
                request: request_config,
                zk_coprocessor_type: IProve::ProofType::SP1ZKP,
                proof: Bytes::from(proof_bytes),
            },
        )
        .abi_encode();
        tracing::info!("Calldata: {}", hex::encode(&calldata));
        // submit proof transaction to Halo contract to verify proof
        match tx_sender.send(calldata.clone()).await {
            Ok((tx_hash, receipt)) => {
                match receipt {
                    Some(receipt) => {
                        tracing::info!("Success: transaction hash: {}", tx_hash);
                        tracing::info!("Failure: transaction receipt: {:#?}", receipt);
                        assert!(true)
                    },
                    None => {
                        tracing::error!("Failure: transaction hash: {}", tx_hash);
                        assert!(false)
                    }
                }
            },
            Err(e) => assert!(false, "Failed to submit proof transaction: {}", e)
        }
    }

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn call_prove_request() {
        let request_config = RequestConfig {
            nonce: Uint::from(94),
            creator: "0x0405399e19220720a5d59cba7fc2b2bb11962532".parse().unwrap(),
            operator: "0x548df1990b444f0b658c838be334149c1ea79833".parse().unwrap(),
            model: "0x682db2fe997945208caa888543ffca2ad2c7edf1ab0b02899b9977e6d18af477".parse().unwrap(),
            fee: Uint::from(0),
            deadline: Uint::from(1745791225),
        };
        tracing::info!("ProveRequest RequestConfig: {:#?}", request_config);

        let output = hex::decode("02550004810000000020a06f00000007010300000000000000000000000000c51e5cb16c461fe29b60394984755325ecd05a9a7a8fb3a116f1c3cf0aca4b0eb9edefb9b404deeaee4b7d454372d17a000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000702000000000000c68518a0ebb42136c12b2275164f8c72f25fa9a34392228687ed6e9caeb9c0f1dbd895e9cf475121c029dc47e70e91fd00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000085e0855a6384fa1c8a6ab36d0dcbfaa11a5753e5a070c08218ae5fe872fcb86967fd2449c29e22e59dc9fec998cb65474a7db64a609c77e85f603c23e9a9fd03bfd9e6b52ce527f774a598e66d58386026cea79b2aea13b81a0b70cfacdec0ca8a4fe048fea22663152ef128853caa5c033cbe66baf32ba1ff7f6b1afc1624c279f50a4cbc522a735ca6f69551e61ef2efb98b5bae8f04d99c50a0174182dca782a2a6f5b891f5a09bd5887bb8904cb0814f6c3d00026953a5c45d8abfd22c8d0000000000000000000000000000000000000000000000000000000000000000a7654f588b28ef3b4833e50f8c2b001d4c67a8164e86b4d1fbf4db149c1f5ac200000000680c04570b2e5424728531e3183fa52906f9ff882ddff3cbccd3b19e2c418bbae9ccf30aa7fa01fc7a25a72b367cd8bd6aed0bb37108920a3292f557465b91fac3a68eb10fa74a3f32c80b978c8ad671395dabf24283eef9091bc3919fd39b9915a87f1adf3061c165c0191e2658256a2855cac9267f179aafb1990c9e918d6452816adf88b0758c525b2f28ee1896907de49511ffb1d919b04bd65b91943be6ebb0a5fe8442f7d95513d4e780cc15c5cd72d9395828137c632877fded3ba0ad43efd4e9").unwrap();
        let proof = hex::decode("11b6a09d074d81b271dd2256c6cbb1be23b67264c4e871cf0d81ca83e676ec03fcb5f1db2a09a910b27807ac4998523f2bf7580dd1fe5e84742bddea572cd1570fcdb95801ecd7af28ebd7afc4c4adb89645807a95b2b68710507828a6c2fcf767cdc8be239e2e278a00ea7dd08a6a346bd34f5865567961244bb3960a6d0322b683ec070f260184519f5c55fb540407c1ec046b86642e0f1300ca2bd7d52b9a56ab4fcf2f20e7f72338d012e7a3e8201a3b87a1dc91efea21ef25705f642ced9652915b075e29c4cf14e408eb9e682ea928a2f5373e39d3d129272ae6d78ee8da8846240a67a5cd373919d0c0b889d4814e21cb9376ee60e59996d23fb5599b3a6295ff").unwrap();
        let proof_bytes = concat_with_length_prefix(output.as_bytes(), proof.as_bytes());
        tracing::info!("ProveRequest Proof (Bytes): {:#?}", Bytes::from(proof_bytes.clone()));

        let tx_sender = TxSender::new(
            "https://sepolia.base.org",
            "0x9E4a45c40e06CE0653C33769138dF48802c1CF1e",
            Some(NamedChain::BaseSepolia),
            None
        ).expect("Failed to create txSender");

        let calldata = IProve::IProveCalls::proveRequest(
            IProve::proveRequestCall {
                request: request_config,
                zk_coprocessor_type: IProve::ProofType::SP1ZKP,
                proof: Bytes::from(proof_bytes),
            },
        )
        .abi_encode();
        tracing::info!("Calldata: {}", hex::encode(&calldata));
        // submit proof transaction to Halo contract to verify proof
        match tx_sender.call(calldata.clone()).await {
            Ok(result) => {
                let (verified, _output) = decode_attestation_ret_data(result.to_vec());
                assert!(verified);
            },
            Err(e) => assert!(false, "Failed to submit proof transaction: {}", e)
        }
    }
}
