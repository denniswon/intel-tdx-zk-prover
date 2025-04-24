use anyhow::Result;

use crate::config::parameter;

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
    let rpc_url = parameter::get("DEFAULT_RPC_URL").parse().expect("Failed to parse RPC URL");
    let provider = ProviderBuilder::new().connect_http(rpc_url);

    let pcs_dao_contract = IPCSDao::new(parameter::get("PCS_DAO_ADDRESS").parse::<Address>().unwrap(), &provider);

    let call_builder = pcs_dao_contract.getCertificateById(ca_id);

    let call_return = call_builder.call().await?;

    let cert = call_return.cert.to_vec();
    let crl = call_return.crl.to_vec();

    Ok((cert, crl))
}
