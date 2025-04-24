#![allow(dead_code)]

pub mod attestation;
pub mod pccs;

use alloy::{
    network::{EthereumWallet, TransactionBuilder},
    primitives::{Address, Bytes, TxHash},
    providers::{Provider, ProviderBuilder},
    rpc::types::{TransactionReceipt, TransactionRequest},
    signers::{k256::ecdsa::SigningKey, local::PrivateKeySigner, utils::secret_key_to_address}
};
use alloy_chains::NamedChain;
use anyhow::Result;

use crate::config::parameter;

pub struct TxSender {
    pub rpc_url: String,
    pub chain: NamedChain,
    pub wallet: EthereumWallet,
    pub contract: Address,
}

impl TxSender {
    /// Creates a new `TxSender`.
    pub fn new(rpc_url: &str, contract: &str, chain: NamedChain) -> Result<Self> {
        let contract = contract.parse::<Address>()?;

        let pk_signer: PrivateKeySigner = parameter::get("NETWORK_PRIVATE_KEY").parse()?;
        let wallet = EthereumWallet::new(pk_signer);

        Ok(TxSender {
            chain,
            rpc_url: rpc_url.to_string(),
            wallet,
            contract,
        })
    }

    /// Sends the transaction
    pub async fn send(&self, calldata: Vec<u8>) -> Result<(TxHash, Option<TransactionReceipt>)> {
        let rpc_url = self.rpc_url.parse()?;

        let provider = ProviderBuilder::new()
            .wallet(self.wallet.clone())
            .with_chain(self.chain)
            .with_cached_nonce_management()
            .with_gas_estimation()
            .connect_http(rpc_url);
        let chain_id = provider.get_chain_id().await?;

        let tx = TransactionRequest::default()
            .with_chain_id(chain_id)
            .with_to(self.contract)
            .with_nonce(0)
            .max_fee_per_gas(20_000_000)
            .max_priority_fee_per_gas(10_000_000)
            .with_gas_limit(10_000_000)
            .with_input(calldata);

        let tx_envelope = tx.build(&self.wallet).await?;
        let tx_hash = tx_envelope.tx_hash().clone();
        let tx = match provider
            .send_tx_envelope(tx_envelope)
            .await {
                Ok(tx) => {
                    tracing::info!("TxSender: Transaction hash: {}", hex::encode(tx_hash));
                    Some(tx)
                },
                Err(e) => {
                    tracing::error!("Failed to send transaction: {} {}", e, hex::encode(tx_hash));
                    None
                }
            };
        match tx {
            Some(tx) => {
                match tx.get_receipt().await {
                    Ok(receipt) => Ok((tx_hash, Some(receipt))),
                    Err(_) => Ok((tx_hash, None))
                }
            },
            None => Ok((tx_hash, None))
        }
    }

    /// Makes a staticcall with the given transaction request
    pub async fn call(&self, calldata: Vec<u8>) -> Result<Bytes> {
        let rpc_url = self.rpc_url.parse()?;
        let provider = ProviderBuilder::new()
            .wallet(&self.wallet)
            .connect_http(rpc_url);

        let tx = TransactionRequest::default()
            .with_to(self.contract)
            .with_input(calldata);

        let call_output = provider.call(tx).await?;

        Ok(call_output)
    }
}

pub fn get_evm_address_from_key(key: &str) -> String {
    let key_slice = hex::decode(key).unwrap();
    let signing_key = SigningKey::from_slice(&key_slice).expect("Invalid key");
    let address = secret_key_to_address(&signing_key);
    address.to_checksum(None)
}
