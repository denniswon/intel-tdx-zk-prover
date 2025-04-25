#![allow(dead_code)]

pub mod chain;
pub mod constants;
pub mod parser;
pub mod utils;
pub mod prove;

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy::{primitives::{Bytes, Uint}, sol_types::SolInterface};
    use alloy_chains::NamedChain;
    use chrono::{TimeZone, Utc};
    use sqlx::types::Uuid;
    use x509_parser::nom::AsBytes;

    use crate::{
        entity::onchain_request::OnchainRequest,
        sp1::chain::{
            attestation::{concat_with_length_prefix, IProve::{self, RequestConfig}},
            TxSender
        }
    };

    use super::*;

    #[tokio::test]
    async fn submit_dummy_proof() {
        let quote = include_bytes!("../../data/quote.hex");
        let proof = prove::prove(quote.to_vec(), None).await.unwrap();
        let request = OnchainRequest {
            id: Uuid::from_str("43c16556-7f70-4aa3-94da-2942ede2e2b2").unwrap(),
            creator_address: "0xCc174c58dFdaF1126f76bE79902535081c798e4E".parse().unwrap(),
            operator_address: "0x548df1990b444F0b658c838bE334149C1eA79833".parse().unwrap(),
            model_id: "0x682db2fe997945208caa888543ffca2ad2c7edf1ab0b02899b9977e6d18af477".parse().unwrap(),
            fee_wei: 0,
            nonce: 23,
            request_id: vec![199, 217, 140, 244, 44, 169, 18, 202, 93, 116, 218, 59, 102, 46, 62, 39, 50, 23, 127, 45, 166, 47, 32, 0, 194, 99, 231, 37, 136, 16, 251, 66],
            deadline: Utc.with_ymd_and_hms(2025, 04, 27, 14, 53, 25).unwrap(),
            is_cancelled: false,
            cancelled_at: None,
            created_at: Utc.with_ymd_and_hms(2025, 04, 24, 14, 53, 46).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2025, 04, 24, 14, 53, 46).unwrap()
        };
        let (success, _output, _, _) = prove::submit_proof(request, proof.proof).await.unwrap();
        assert_eq!(success, false);
    }

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn submit_proof() {
        let request_config = RequestConfig {
            nonce: Uint::from(23),
            creator: "0xCc174c58dFdaF1126f76bE79902535081c798e4E".parse().unwrap(),
            operator: "0x548df1990b444F0b658c838bE334149C1eA79833".parse().unwrap(),
            model: "0x682db2fe997945208caa888543ffca2ad2c7edf1ab0b02899b9977e6d18af477".parse().unwrap(),
            fee: Uint::from(0),
            deadline: Uint::from(1745765605),
        };
        let output = hex::decode("02550004810000000020a06f00000007010300000000000000000000000000c51e5cb16c461fe29b60394984755325ecd05a9a7a8fb3a116f1c3cf0aca4b0eb9edefb9b404deeaee4b7d454372d17a000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000702000000000000c68518a0ebb42136c12b2275164f8c72f25fa9a34392228687ed6e9caeb9c0f1dbd895e9cf475121c029dc47e70e91fd00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000085e0855a6384fa1c8a6ab36d0dcbfaa11a5753e5a070c08218ae5fe872fcb86967fd2449c29e22e59dc9fec998cb65474a7db64a609c77e85f603c23e9a9fd03bfd9e6b52ce527f774a598e66d58386026cea79b2aea13b81a0b70cfacdec0ca8a4fe048fea22663152ef128853caa5c033cbe66baf32ba1ff7f6b1afc1624c279f50a4cbc522a735ca6f69551e61ef2efb98b5bae8f04d99c50a0174182dca782a2a6f5b891f5a09bd5887bb8904cb0814f6c3d00026953a5c45d8abfd22c8d0000000000000000000000000000000000000000000000000000000000000000c7d98cf42ca912ca5d74da3b662e3e2732177f2da62f2000c263e7258810fb4200000000680bba730b2e5424728531e3183fa52906f9ff882ddff3cbccd3b19e2c418bbae9ccf30aa7fa01fc7a25a72b367cd8bd6aed0bb37108920a3292f557465b91fac3a68eb10fa74a3f32c80b978c8ad671395dabf24283eef9091bc3919fd39b9915a87f1adf3061c165c0191e2658256a2855cac9267f179aafb1990c9e918d6452816adf88b0758c525b2f28ee1896907de49511ffb1d919b04bd65b91943be6ebb0a5fe8442f7d95513d4e780cc15c5cd72d9395828137c632877fded3ba0ad43efd4e9").unwrap();
        let proof = hex::decode("11b6a09d1a815df41f392e52a569124df59a85b2e358210426c3b44396d15ed6f04262ee1bd77ed9d09c3dbf39da52cfa66872109c174475c264190d14c11a16d57e1ad825889adc3c72e443b82a9335f556c73c2f02aaaad0aa0ed37a26e2464eef119f16300858225e8036dbe5886be0ff5161311561e8df99c17bc78130cf1442ca2328f9923bedd48de14e8b066d70f95a3b6661098d9cd19e954608bc5824d5fff71e184b4695e40b6a4d1a0a8fe2907bc78a4f18d43de6c17770e496736fabf80614a6e5952738e3bb7216adf576642f1d45f6eabbe36ddfadfc83731550551a1825a843e827e218ba64e48d6afa6a3b27e7f79c469790c90aa218b903d6845eaf").unwrap();
        let proof_bytes = concat_with_length_prefix(output.as_bytes(), proof.as_bytes());
        tracing::info!("ProveRequest Proof (Bytes): {:#?}", proof_bytes);

        let tx_sender = TxSender::new(
            "https://mainnet.base.org",
            "0x9E4a45c40e06CE0653C33769138dF48802c1CF1e",
            Some(NamedChain::Base),
            "0xf32bfe39d5acee9aa3294a1d2927c34f92d862cf115ec1e6478d4e2789911c22"
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
}
