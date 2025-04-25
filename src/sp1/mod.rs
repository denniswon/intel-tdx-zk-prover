#![allow(dead_code)]

pub mod chain;
pub mod constants;
pub mod parser;
pub mod utils;
pub mod prove;

#[cfg(test)]
mod tests {
    use alloy::{primitives::{Bytes, Uint}, sol_types::SolInterface};
    use alloy_chains::NamedChain;
    use x509_parser::nom::AsBytes;

    use crate::sp1::chain::{
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
