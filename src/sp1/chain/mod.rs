#![allow(dead_code)]

pub mod attestation;
pub mod pccs;

use alloy::{
    network::{Ethereum, EthereumWallet, TransactionBuilder},
    primitives::{Address, Bytes, TxHash},
    providers::{PendingTransactionBuilder, Provider, ProviderBuilder},
    rpc::types::{TransactionReceipt, TransactionRequest},
    signers::{k256::ecdsa::SigningKey, local::PrivateKeySigner, utils::secret_key_to_address},
};
use alloy_chains::NamedChain;
use anyhow::Result;

pub struct TxSender {
    pub rpc_url: String,
    pub chain: NamedChain,
    pub wallet: EthereumWallet,
    pub account: Address,
    pub contract: Address,
}

impl TxSender {
    /// Creates a new `TxSender`.
    pub fn new(rpc_url: &str, contract: &str, chain: NamedChain, pk: &str) -> Result<Self> {
        let contract = contract.parse::<Address>()?;

        let pk_signer: PrivateKeySigner = pk.parse()?;
        let account = pk_signer.address();
        let wallet = EthereumWallet::new(pk_signer);

        Ok(TxSender {
            chain,
            rpc_url: rpc_url.to_string(),
            wallet,
            account,
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
        
        let tx_request = TransactionRequest::default()
            .with_chain_id(chain_id)
            .with_to(self.contract)
            .with_from(self.account)
            .with_input(calldata);

        let mut nonce = provider.get_transaction_count(self.account).await?;
        let mut multiplier = 1.0;
        let mut max_retries = 2;

        let mut pending_tx: Option<PendingTransactionBuilder<Ethereum>> = None;
        let mut transaction_hash: Option<TxHash> = None;
        loop {
            nonce += 1;
            let gas_limit = provider.estimate_gas(tx_request.clone()).await?;
            let max_fee_per_gas = provider.estimate_eip1559_fees().await?;

            let tx = tx_request.clone()
                .with_nonce(nonce)
                .max_fee_per_gas((max_fee_per_gas.max_fee_per_gas as f64 * multiplier) as u128)
                .max_priority_fee_per_gas((max_fee_per_gas.max_priority_fee_per_gas as f64 * multiplier) as u128)
                .with_gas_limit((gas_limit as f64 * multiplier) as u64);

            let tx_envelope = tx.build(&self.wallet).await?;
            let tx_hash = tx_envelope.tx_hash().clone();
            
            let tx = match provider
                .send_tx_envelope(tx_envelope.clone())
                .await {
                    Ok(tx) => {
                        tracing::info!("TxSender: Transaction hash: {}", tx_hash);
                        tracing::debug!("TxSender: Transaction: {:#?}", tx_envelope.into_typed_transaction());
                        Some(tx)
                    },
                    Err(e) => {
                        tracing::error!("Failed to send transaction: {} {}", e, tx_hash);
                        if e.as_error_resp().unwrap().code == -32603 {
                            tracing::info!("Retrying transaction");
                            multiplier *= 1.1;
                            max_retries -= 1;
                        } else {
                            tracing::info!("Not retriable error. Skipping retrying transaction: {} {}", e, tx_hash);
                            max_retries = 0;
                        }
                        None
                    }
                };

            if tx.is_some() {
                pending_tx = tx;
                transaction_hash = Some(tx_hash);
                break;
            } else if max_retries == 0 {
                break;
            }
        };

        match pending_tx {
            Some(tx) => {
                match tx.get_receipt().await {
                    Ok(receipt) => Ok((transaction_hash.unwrap(), Some(receipt))),
                    Err(_) => Ok((transaction_hash.unwrap(), None))
                }
            },
            None => Ok((transaction_hash.unwrap(), None))
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
