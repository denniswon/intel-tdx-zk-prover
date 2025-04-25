use std::str::FromStr;

use anyhow::Result;

use crate::sp1::constants::AUTOMATA_DEFAULT_RPC_URL;
use crate::{config::parameter, sp1::constants::AUTOMATA_FMSPC_TCB_DAO_ADDRESS};
use crate::sp1::utils::remove_prefix_if_found;

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
    let verify_only = parameter::get("VERIFY_ONLY");
    let rpc_url = if verify_only == "true" {
        AUTOMATA_DEFAULT_RPC_URL.parse().expect("Failed to parse RPC URL")
    } else {
        parameter::get("DEFAULT_RPC_URL").parse().expect("Failed to parse RPC URL")
    };
    let provider = ProviderBuilder::new().connect_http(rpc_url);

    let fmspc_tcb_dao_contract =
        IFmspcTcbDao::new(
            if verify_only == "true" {
                Address::from_str(AUTOMATA_FMSPC_TCB_DAO_ADDRESS).unwrap()
            } else {
                parameter::get("FMSPC_TCB_DAO_ADDRESS").parse::<Address>().unwrap()
            },
            &provider
        );

    let call_builder = fmspc_tcb_dao_contract.getTcbInfo(
        U256::from(tcb_type),
        String::from(fmspc),
        U256::from(version),
    );

    let call_return = call_builder.call().await?;
    let tcb_info_str = call_return.tcbInfoStr;
    let signature_bytes = call_return.signature;

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
