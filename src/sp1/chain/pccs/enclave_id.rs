use std::str::FromStr;

use anyhow::Result;

use crate::sp1::constants::AUTOMATA_ENCLAVE_ID_DAO_ADDRESS;
use crate::{config::parameter, sp1::constants::AUTOMATA_DEFAULT_RPC_URL};
use crate::sp1::utils::remove_prefix_if_found;

use alloy::{
    primitives::{Address, U256},
    providers::ProviderBuilder,
    sol,
};

sol! {
    #[sol(rpc)]
    interface IEnclaveIdentityDao {
        #[derive(Debug)]
        struct EnclaveIdentityJsonObj {
            string identityStr;
            bytes signature;
        }

        #[derive(Debug)]
        function getEnclaveIdentity(uint256 id, uint256 version) returns (EnclaveIdentityJsonObj memory enclaveIdObj);
    }
}

#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum EnclaveIdType {
    QE,
    QVE,
    TDQE,
}

pub async fn get_enclave_identity(id: EnclaveIdType, version: u32) -> Result<Vec<u8>> {
    let verify_only = parameter::get("VERIFY_ONLY");
    let rpc_url = if verify_only == "true" {
        AUTOMATA_DEFAULT_RPC_URL.parse().expect("Failed to parse RPC URL")
    } else {
        parameter::get("DEFAULT_RPC_URL").parse().expect("Failed to parse RPC URL")
    };
    let provider = ProviderBuilder::new().connect_http(rpc_url);

    let enclave_id_dao_address = if verify_only == "true" {
        Address::from_str(AUTOMATA_ENCLAVE_ID_DAO_ADDRESS).unwrap()
    } else {
        parameter::get("ENCLAVE_ID_DAO_ADDRESS").parse::<Address>().unwrap()
    };
    let enclave_id_dao_contract = IEnclaveIdentityDao::new(
        enclave_id_dao_address,
        &provider,
    );

    let enclave_id_type_uint256 = match id {
        EnclaveIdType::QE => U256::from(0),
        EnclaveIdType::QVE => U256::from(1),
        EnclaveIdType::TDQE => U256::from(2),
    };

    let call_builder =
        enclave_id_dao_contract.getEnclaveIdentity(enclave_id_type_uint256, U256::from(version));

    let call_return = call_builder.call().await?;

    let identity_str = call_return.identityStr;
    let signature_bytes = call_return.signature;

    if identity_str.is_empty() || signature_bytes.len() == 0 {
        return Err(anyhow::Error::msg(format!(
            "QEIdentity for ID: {:?}; Version: {} is missing and must be upserted to on-chain pccs",
            id, version
        )));
    }

    let signature = signature_bytes.to_string();

    let ret_str = format!(
        "{{\"enclaveIdentity\": {}, \"signature\": \"{}\"}}",
        identity_str,
        remove_prefix_if_found(signature.as_str())
    );

    let ret = ret_str.into_bytes();
    Ok(ret)
}
