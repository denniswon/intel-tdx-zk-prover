use std::str::FromStr;

use anyhow::Result;

use crate::chain::constants::AUTOMATA_ENCLAVE_ID_DAO_ADDRESS;
use crate::config::parameter;
use crate::chain::utils::remove_prefix_if_found;

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

    let enclave_id_dao_address = match verify_only {
        true => Address::from_str(AUTOMATA_ENCLAVE_ID_DAO_ADDRESS).unwrap(),
        false => parameter::get(
            "ENCLAVE_ID_DAO_ADDRESS", Some("0xd74e880029cd3b6b434f16bea5f53a06989458ee")
        ).parse::<Address>().unwrap()
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

    let identity_str = call_return.enclaveIdObj.identityStr;
    let signature_bytes = call_return.enclaveIdObj.signature;

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
