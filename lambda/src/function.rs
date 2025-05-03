use std::{str::FromStr, sync::Arc};
use tdx_prover::{
    config::{
        database::{Database, DatabaseTrait},
        parameter,
    },
    entity::quote::{ProofType, TdxQuoteStatus},
    error::{db_error::DbError, quote_error::QuoteError},
    repository::{quote_repository::QuoteRepositoryTrait, request_repository::OnchainRequestRepositoryTrait},
    state::{quote_state::QuoteState, request_state::RequestState}, zk,
};
use aws_lambda_events::eventbridge::EventBridgeEvent;
use hex::FromHex;
use lambda_runtime::{Error, LambdaEvent};
use serde_json::Value;

pub(crate) async fn handler(event: LambdaEvent<EventBridgeEvent>) -> Result<(), Error> {
    // initialize tracing for logging
    tracing_subscriber::fmt().init();
    tracing::info!("Event: {:?}", event);
    parameter::init();

    let request_id_hex = event.payload.detail.get("request_id").unwrap().as_str().unwrap();
    tracing::info!("Request ID hex: {}", request_id_hex);
    let request_id = Vec::from_hex(request_id_hex.strip_prefix("0x").unwrap_or(request_id_hex))?;

    let default_proof_type = Value::String("sp1".to_string());
    let proof_type_str = event.payload.detail.get("proof_type").unwrap_or(&default_proof_type).as_str().unwrap();
    tracing::info!("Proof type: {}", proof_type_str);
    let proof_type = ProofType::from_str(proof_type_str.to_lowercase().as_str()).unwrap();

    let db_conn = Arc::new(
        Database::init()
            .await
            .unwrap_or_else(|e| panic!("Database error: {}", e)),
    );

    let quote_state = QuoteState::new(&db_conn);
    let request_state = RequestState::new(&db_conn);

    let onchain_request = request_state.request_repo.find_by_request_id(request_id).await.map_err(|e| {
        tracing::error!("Failed to fetch onchain request: {}", e);
        DbError::SomethingWentWrong("Failed to fetch onchain request".to_string())
    })?;

    tracing::info!("Onchain request found: {:#?}", onchain_request);

    let attestation = quote_state.quote_repo.find_by_onchain_request_id(onchain_request.id).await.map_err(|e| {
        tracing::error!("Failed to fetch attestation: {}", e);
        DbError::SomethingWentWrong("Failed to fetch attestation".to_string())
    })?;

    let quote_id = attestation.id;
    tracing::info!("Attestation found for request ID: {} {}", request_id_hex, attestation.status);
    let proof = zk::prove(attestation.quote, proof_type, None).await
        .map_err(|e| {
            tracing::error!("Failed to generate proof for request ID: {:?} {}", request_id_hex, e.to_string());
            QuoteError::Prove
        })?;
    
    tracing::info!("Proof generated for request ID: {:?}", request_id_hex);
    
    // only verify proof in dev because in lambda, filesystem is not writable
    if std::env::var("ENV").unwrap_or("dev".to_string()) != "prod" {
        tracing::info!("Verifying proof...");
        
        zk::verify_proof(&proof.proof).await.map_err(|e| {
            tracing::error!("Failed to verify proof: {}", e);
            QuoteError::VerifyProof
        })?;
        tracing::info!("Successfully verified proof.");
    }

    let verify_only = parameter::get("VERIFY_ONLY").to_lowercase() == "true";

    let (verified, raw_verified_output, tx_hash, response) =
        zk::submit_proof(onchain_request, proof_type, proof.proof, Some(verify_only)).await
            .map_err(|e| {
                tracing::error!("Failed to submit proof: {}", e);
                QuoteError::SubmitProof
            })?;

    tracing::info!(
        "Proof submitted for request ID: {} verified: {} raw_verified_output: {}",
        request_id_hex, verified, hex::encode(&raw_verified_output)
    );

    if response.is_some() {
        tracing::info!("Submit proof response: {:#?}", response);
        let response = response.unwrap();
        tracing::info!("Transaction hash: {}", hex::encode(&response.transaction_hash));

        // Update onchain request status
        quote_state.quote_repo.update_status(
            quote_id,
            proof_type,
            TdxQuoteStatus::Failure,
            Some(response.transaction_hash.to_vec()),
            None,
        ).await.map_err(|e| {
            tracing::error!("Failed to update quote status on success: {}", e);
            QuoteError::UpdateStatusOnSuccess
        })?;

        tracing::info!("tdx_quote updated successfully {quote_id} {}", TdxQuoteStatus::Success);
    } else if tx_hash.is_some() {
        tracing::info!("Transaction hash: {}", hex::encode(&tx_hash.unwrap().to_vec()));
        // Update onchain request status
        quote_state.quote_repo.update_status(
            quote_id,
            proof_type,
            TdxQuoteStatus::Failure,
            Some(tx_hash.unwrap().to_vec()),
            None,
        ).await.map_err(|e| {
            tracing::error!("Failed to update quote status on failure: {}", e);
            QuoteError::UpdateStatusOnFailure
        })?;

        tracing::info!("tdx_quote updated successfully {quote_id} {}", TdxQuoteStatus::Failure);
    }

    Ok(())
}
