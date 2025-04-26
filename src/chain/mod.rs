#![allow(dead_code)]

pub mod attestation;
pub mod pccs;
pub mod constants;
pub mod utils;

use std::{cmp::max, thread, time::Duration};

use alloy::{
    eips::eip1559::Eip1559Estimation,
    network::{Ethereum, TransactionBuilder},
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
    pub chain: Option<NamedChain>,
    pub signer: PrivateKeySigner,
    pub account: Address,
    pub contract: Address,
}

impl TxSender {
    /// Creates a new `TxSender`.
    pub fn new(rpc_url: &str, contract: &str, chain: Option<NamedChain>, pk: Option<&str>) -> Result<Self> {
        let contract = contract.parse::<Address>()?;

        let signer: PrivateKeySigner = if let Some(pk) = pk {
            pk.parse()?
        } else {
            PrivateKeySigner::random()
        };
        let account = signer.address();

        Ok(TxSender {
            chain,
            rpc_url: rpc_url.to_string(),
            signer,
            account,
            contract,
        })
    }

    /// Sends the transaction
    pub async fn send(&self, calldata: Vec<u8>) -> Result<(TxHash, Option<TransactionReceipt>)> {
        let rpc_url = self.rpc_url.parse()?;

        let provider = ProviderBuilder::new()
            .wallet(self.signer.clone())
            .connect_http(rpc_url);

        let tx_request = TransactionRequest::default()
            .with_to(self.contract)
            .with_from(self.account)
            .with_input(calldata);

        let builder = provider.send_transaction(tx_request).await?
            .with_required_confirmations(1)
            .with_timeout(Some(std::time::Duration::from_secs(60)));
        let tx_hash = *builder.tx_hash();
        tracing::info!("TxSender: transaction hash: {}", tx_hash);

        match provider.get_transaction_by_hash(tx_hash).await {
            Ok(pending_tx) => {
                tracing::info!("TxSender: proof tx sent: {:#?}", pending_tx);
                Some(pending_tx)
            },
            Err(e) => {
                tracing::warn!("TxSender: failed to get transaction by hash: {} {}", tx_hash, e);
                None
            }
        };
        
        tracing::info!("TxSender: waiting for transaction receipt");
        match builder.get_receipt().await {
            Ok(receipt) => {
                tracing::info!("TxSender: transaction receipt received: {:#?}", receipt);
                Ok((tx_hash, Some(receipt)))
            },
            Err(e) => {
                tracing::error!("TxSender: failed to get transaction receipt: {} {}", tx_hash, e);
                Ok((tx_hash, None))
            }
        }
    }

    /// Sends raw transaction with retry
    pub async fn send_raw(&self, calldata: Vec<u8>, max_retries: Option<usize>) -> Result<(TxHash, Option<TransactionReceipt>)> {
        let rpc_url = self.rpc_url.parse()?;
        let mut max_retries = max_retries.unwrap_or(0);

        let provider = match self.chain {
            Some(chain) => ProviderBuilder::new()
                .wallet(self.signer.clone())
                .with_chain(chain)
                .with_cached_nonce_management()
                .with_gas_estimation()
                .connect_http(rpc_url),
            None => ProviderBuilder::new()
                .wallet(self.signer.clone())
                .with_cached_nonce_management()
                .with_gas_estimation()
                .connect_http(rpc_url),
        };

        let chain_id = provider.get_chain_id().await?;
        
        let tx_request = TransactionRequest::default()
            .with_chain_id(chain_id)
            .with_to(self.contract)
            .with_from(self.account)
            .with_input(calldata);

        let mut nonce = provider.get_transaction_count(self.account).await? + 1;
        let mut gas_limit = match provider.estimate_gas(tx_request.clone()).await {
            Ok(gas_limit) => max((gas_limit as f64 * 1.5) as u64, 10_000_000),
            Err(e) => {
                tracing::warn!("Failed to estimate gas: {}", e);
                10_000_000
            }
        };
        let max_fee_per_gas = match provider.estimate_eip1559_fees().await {
            Ok(max_fee_per_gas) => {
                Eip1559Estimation {
                    max_fee_per_gas: max((max_fee_per_gas.max_fee_per_gas as f64 * 1.5) as u128, 5_000_000),
                    max_priority_fee_per_gas: max((max_fee_per_gas.max_priority_fee_per_gas as f64 * 1.5) as u128, 500_000)
                }
            },
            Err(e) => {
                tracing::warn!("Failed to estimate max fee per gas: {}", e);
                Eip1559Estimation {
                    max_fee_per_gas: 5_000_000,
                    max_priority_fee_per_gas: 500_000
                }
            }
        };
        let mut max_priority_fee_per_gas = max_fee_per_gas.max_priority_fee_per_gas;
        let mut max_fee_per_gas = max_fee_per_gas.max_fee_per_gas;

        let multiplier = 1.1;

        let mut pending_tx: Option<PendingTransactionBuilder<Ethereum>> = None;
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

            let builder = match provider.send_transaction(tx).await {
                Ok(tx) => {
                    tracing::info!("TxSender: Transaction hash: {}", *tx.tx_hash());
                    Some(tx.with_timeout(Some(Duration::from_secs(60))))
                },
                Err(e) => {
                    tracing::error!("TxSender: Failed to send transaction: {}", e);
                    nonce += 1;
                    max_fee_per_gas = (max_fee_per_gas as f64 * multiplier) as u128;
                    max_priority_fee_per_gas = (max_priority_fee_per_gas as f64 * multiplier) as u128;
                    gas_limit = (gas_limit as f64 * multiplier) as u64;
                    max_retries -= 1;
                    None
                }
            };

            if let Some(builder) = builder {
                pending_tx = Some(builder);
                break;
            } else if max_retries == 0 {
                break;
            }

            tracing::info!("TxSender: Retrying transaction. {} retries left", max_retries);
            let backoff = Duration::from_millis(500);
            thread::sleep(backoff);
        }

        match pending_tx {
            Some(tx) => {
                let tx_hash = *tx.tx_hash();
                tracing::info!("TxSender: Waiting for transaction receipt: {}", tx_hash);
                match tx.get_receipt().await {
                    Ok(receipt) => {
                        tracing::info!("TxSender: Transaction receipt received: {}", tx_hash);
                        Ok((tx_hash, Some(receipt)))
                    },
                    Err(e) => {
                        tracing::error!("TxSender: Failed to get transaction receipt: {} {}", tx_hash, e);
                        Ok((tx_hash, None))
                    }
                }
            },
            None => {
                tracing::error!("TxSender: Failed to send transaction. Aborting");
                Err(anyhow::anyhow!("Failed to send transaction"))
            }
        }
    }

    /// Makes a staticcall with the given transaction request
    pub async fn call(&self, calldata: Vec<u8>) -> Result<Bytes> {
        let rpc_url = self.rpc_url.parse()?;
        let provider = ProviderBuilder::new()
            .wallet(self.signer.clone())
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
