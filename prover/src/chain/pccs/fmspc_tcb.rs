use std::str::FromStr;

use anyhow::Result;

use crate::{config::parameter, chain::constants::AUTOMATA_FMSPC_TCB_DAO_ADDRESS};
use crate::chain::utils::remove_prefix_if_found;

use alloy::{
    primitives::{Address, U256},
    providers::ProviderBuilder,
    sol,
};

sol! {
    #[sol(rpc)]
    interface IFmspcTcbDao {
        #[derive(Debug)]
        struct TcbInfoJsonObj {
            string tcbInfoStr;
            bytes signature;
        }

        #[derive(Debug)]
        function getTcbInfo(uint256 tcbType, string calldata fmspc, uint256 version) returns (TcbInfoJsonObj memory tcbObj);
    }
}

pub async fn get_tcb_info(tcb_type: u8, fmspc: &str, version: u32) -> Result<Vec<u8>> {
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

    let fmspc_tcb_dao_contract =
        IFmspcTcbDao::new(
            match verify_only {
                true => Address::from_str(AUTOMATA_FMSPC_TCB_DAO_ADDRESS).unwrap(),
                false => parameter::get(
                    "FMSPC_TCB_DAO_ADDRESS",
                    Some("0xd3A3f34E8615065704cCb5c304C0cEd41bB81483")
                ).parse::<Address>().unwrap()
            },
            &provider
        );

    let call_builder = fmspc_tcb_dao_contract.getTcbInfo(
        U256::from(tcb_type),
        String::from(fmspc),
        U256::from(version),
    );

    let call_return = call_builder.call().await?;
    let tcb_info_str = call_return.tcbObj.tcbInfoStr;
    let signature_bytes = call_return.tcbObj.signature;

    if tcb_info_str.is_empty() || signature_bytes.len() == 0 {
        return Err(anyhow::Error::msg(format!(
            "TCBInfo for FMSPC: {}; Version: {} is missing and must be upserted to on-chain pccs",
            fmspc, version
        )));
    }

    let signature = signature_bytes.to_string();

    let ret_str = format!(
        "{{\"tcbInfo\": {}, \"signature\": \"{}\"}}",
        tcb_info_str,
        remove_prefix_if_found(signature.as_str())
    );

    let ret = ret_str.into_bytes();
    Ok(ret)
}
