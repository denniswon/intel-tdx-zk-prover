use std::str::FromStr;

use anyhow::Result;

use crate::{config::parameter, chain::constants::AUTOMATA_PCS_DAO_ADDRESS};

use alloy::{primitives::Address, providers::ProviderBuilder, sol};

sol! {
    #[sol(rpc)]
    interface IPCSDao {
        #[derive(Debug)]
        enum CA {
            ROOT,
            PROCESSOR,
            PLATFORM,
            SIGNING
        }

        #[derive(Debug)]
        function getCertificateById(CA ca) external view returns (bytes memory cert, bytes memory crl);
    }
}

pub async fn get_certificate_by_id(ca_id: IPCSDao::CA) -> Result<(Vec<u8>, Vec<u8>)> {
    let verify_only = parameter::get("VERIFY_ONLY", Some("false")) == "true";
    let rpc_url = match verify_only {
        true => parameter::get(
            "AUTOMATA_DEFAULT_RPC_URL", Some("https://1rpc.io/ata/testnet")
        ).parse().expect("Failed to parse RPC URL"),
        false => parameter::get(
            "DEFAULT_RPC_URL", Some("https://mainnet.base.org")
        ).parse().expect("Failed to parse RPC URL")
    };
    let provider = ProviderBuilder::new().on_http(rpc_url);

    let pcs_dao_contract = IPCSDao::new(
        match verify_only {
            true => Address::from_str(AUTOMATA_PCS_DAO_ADDRESS).unwrap(),
            false => parameter::get(
                "PCS_DAO_ADDRESS",
                Some("0xB270cD8550DA117E3accec36A90c4b0b48daD342")
            ).parse::<Address>().unwrap()
        },
        &provider
    );

    let call_builder = pcs_dao_contract.getCertificateById(ca_id);

    let call_return = call_builder.call().await?;

    let cert = call_return.cert.to_vec();
    let crl = call_return.crl.to_vec();

    Ok((cert, crl))
}
