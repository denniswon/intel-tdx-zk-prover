#![allow(dead_code)]

pub mod attestation;
pub mod pccs;

use std::{cmp::max, thread, time::Duration};

use alloy::{
    eips::eip1559::Eip1559Estimation,
    network::{Ethereum, EthereumWallet, TransactionBuilder},
    primitives::{Address, Bytes, TxHash},
    providers::{PendingTransactionBuilder, Provider, ProviderBuilder},
    rpc::types::{TransactionReceipt, TransactionRequest},
    signers::{k256::ecdsa::SigningKey, local::PrivateKeySigner, utils::secret_key_to_address},
};
use alloy_chains::NamedChain;
use anyhow::Result;
use rand::prelude::*;

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

        let mut nonce = provider.get_transaction_count(self.account).await? + 1;
        let mut gas_limit = match provider.estimate_gas(tx_request.clone()).await {
            Ok(gas_limit) => max((gas_limit as f64 * 1.5) as u64, 20_000_000),
            Err(e) => {
                tracing::warn!("Failed to estimate gas: {}", e);
                20_000_000
            }
        };
        let max_fee_per_gas = match provider.estimate_eip1559_fees().await {
            Ok(max_fee_per_gas) => {
                Eip1559Estimation {
                    max_fee_per_gas: max((max_fee_per_gas.max_fee_per_gas as f64 * 1.5) as u128, 10_000_000),
                    max_priority_fee_per_gas: max((max_fee_per_gas.max_priority_fee_per_gas as f64 * 1.5) as u128, 1_000_000)
                }
            },
            Err(e) => {
                tracing::warn!("Failed to estimate max fee per gas: {}", e);
                Eip1559Estimation {
                    max_fee_per_gas: 10_000_000,
                    max_priority_fee_per_gas: 1_000_000
                }
            }
        };
        let mut max_priority_fee_per_gas = max_fee_per_gas.max_priority_fee_per_gas;
        let mut max_fee_per_gas = max_fee_per_gas.max_fee_per_gas;

        let multiplier = 1.2;
        let mut max_retries = 3;

        let mut pending_tx: Option<PendingTransactionBuilder<Ethereum>> = None;
        let mut transaction_hash: Option<TxHash> = None;

        loop {
            nonce += rand::rng().random_range(..5u64);
            tracing::info!("Account nonce: {}", nonce);
            tracing::info!("Max fee per gas: {:#?}", max_fee_per_gas);
            tracing::info!("Max priority fee per gas: {:#?}", max_priority_fee_per_gas);
            tracing::info!("Gas limit: {}", gas_limit);

            let tx = tx_request.clone()
                .with_nonce(nonce)
                .max_fee_per_gas(max_fee_per_gas)
                .max_priority_fee_per_gas(max_priority_fee_per_gas)
                .with_gas_limit(gas_limit);

            let tx_envelope = tx.build(&self.wallet).await?;
            let tx_hash = tx_envelope.tx_hash().clone();

            let tx = match provider
                .send_tx_envelope(tx_envelope.clone())
                .await {
                    Ok(tx) => {
                        tracing::info!("TxSender: Transaction hash: {}", tx_hash);
                        tracing::info!("TxSender: Transaction: {:#?}", tx_envelope.into_typed_transaction());
                        Some(tx.with_timeout(Some(Duration::from_secs(120))))
                    },
                    Err(e) => {
                        tracing::error!("TxSender: Failed to send transaction: {} {}", e, tx_hash);
                        if e.as_error_resp().unwrap().code == -32603 {
                            tracing::info!("TxSender: Retrying transaction");

                            nonce += 1;
                            max_fee_per_gas = (max_fee_per_gas as f64 * multiplier) as u128;
                            max_priority_fee_per_gas = (max_priority_fee_per_gas as f64 * multiplier) as u128;
                            gas_limit = (gas_limit as f64 * multiplier) as u64;

                            max_retries -= 1;

                            let backoff = Duration::from_millis(500);
                            thread::sleep(backoff);
                        } else {
                            tracing::info!("TxSender: Not retriable error. Skipping retrying transaction: {} {}", e, tx_hash);
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
        }

        match pending_tx {
            Some(tx) => {
                tracing::info!("TxSender: Waiting for transaction receipt: {}", transaction_hash.unwrap());
                match tx.get_receipt().await {
                    Ok(receipt) => {
                        tracing::info!("TxSender: Transaction receipt received: {}", transaction_hash.unwrap());
                        Ok((transaction_hash.unwrap(), Some(receipt)))
                    },
                    Err(e) => {
                        tracing::error!("TxSender: Failed to get transaction receipt: {} {}", transaction_hash.unwrap(), e);
                        Ok((transaction_hash.unwrap(), None))
                    }
                }
            },
            None => {
                tracing::error!("TxSender: Failed to send transaction. Aborting: {}", transaction_hash.unwrap());
                Ok((transaction_hash.unwrap(), None))
            }
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
