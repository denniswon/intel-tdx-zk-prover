#![allow(dead_code)]

pub mod attestation;
pub mod pccs;

use alloy::{
    network::{EthereumWallet, TransactionBuilder},
    primitives::{Address, Bytes, TxHash},
    providers::{Provider, ProviderBuilder},
    rpc::types::{TransactionReceipt, TransactionRequest},
    signers::{k256::ecdsa::SigningKey, utils::secret_key_to_address}
};
use alloy_chains::NamedChain;
use anyhow::Result;

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

        Ok(TxSender {
            chain,
            rpc_url: rpc_url.to_string(),
            wallet: EthereumWallet::default(),
            contract,
        })
    }

    /// Sends the transaction
    pub async fn send(&self, calldata: Vec<u8>) -> Result<(TxHash, Option<TransactionReceipt>)> {
        let rpc_url = self.rpc_url.parse()?;
        let provider = ProviderBuilder::new()
            .with_chain(self.chain)
            .wallet(&self.wallet)
            .on_http(rpc_url);

        let tx = TransactionRequest::default()
            .with_to(self.contract)
            .with_input(calldata);

        let tx = provider
            .send_transaction(tx.clone())
            .await?;
        let tx_hash = tx.tx_hash().clone();
        match tx.get_receipt().await {
            Ok(receipt) => Ok((tx_hash, Some(receipt))),
            Err(_) => Ok((tx_hash, None))
        }
    }

    /// Makes a staticcall with the given transaction request
    pub async fn call(&self, calldata: Vec<u8>) -> Result<Bytes> {
        let rpc_url = self.rpc_url.parse()?;
        let provider = ProviderBuilder::new()
            .with_chain(self.chain)
            .wallet(&self.wallet)
            .on_http(rpc_url);

        let tx = TransactionRequest::default()
            .with_to(self.contract)
            .with_input(calldata);

        let call_output = provider.call(&tx).await?;

        Ok(call_output)
    }
}

pub fn get_evm_address_from_key(key: &str) -> String {
    let key_slice = hex::decode(key).unwrap();
    let signing_key = SigningKey::from_slice(&key_slice).expect("Invalid key");
    let address = secret_key_to_address(&signing_key);
    address.to_checksum(None)
}
