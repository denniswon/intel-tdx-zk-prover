use std::sync::Arc;

use crate::{
    config::{
        database::{Database, DatabaseTrait},
        parameter,
    }, entity::quote::{ProofType, TdxQuoteStatus}, error::attestation_error::AttestationError, repository::{onchain_request_repository::OnchainRequestRepositoryTrait, quote_repository::QuoteRepositoryTrait}, sp1::prove, state::attestation_state::AttestationState
};
use aws_lambda_events::eventbridge::EventBridgeEvent;
use lambda_runtime::{Error, LambdaEvent};
use sqlx::types::Uuid;
use tracing::Level;

pub(crate) async fn handler(event: LambdaEvent<EventBridgeEvent>) -> Result<(), Error> {
    tracing::debug!("Event: {:?}", event);
    parameter::init();
    // initialize tracing for logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // Extract some useful information from the request
    let payload = event.payload;
    tracing::debug!("Payload: {:?}", payload);

    let request_id = payload.detail.get("request_id").unwrap();
    tracing::debug!("Request ID: {}", request_id);
    let request_id = Uuid::parse_str(request_id.as_str().unwrap()).unwrap();

    let db_conn = Arc::new(
        Database::init()
            .await
            .unwrap_or_else(|e| panic!("Database error: {}", e)),
    );

    let state = AttestationState::new(&db_conn);

    let attestation = state.quote_repo.find_by_onchain_request_id(request_id).await;
    let mut quote_id: Uuid = Uuid::nil();
    let result = match attestation {
        Ok(attestation) => {
            quote_id = attestation.id;
            tracing::info!("Attestation found for request ID: {} {}", request_id, attestation.status);
            let proof = prove::prove(attestation.quote.clone(), None).await;
            match proof {
                Ok(proof) => {
                    tracing::info!("Proof generated for request ID: {}", request_id);
                    Ok(proof)
                }
                Err(e) => {
                    tracing::error!("Failed to generate proof for request ID: {} {}", request_id, e.to_string());
                    Err(Box::new(AttestationError::Invalid))
                }
            }
        }
        _ => {
            tracing::error!("Attestation not found for request ID: {}", request_id);
            Err(Box::new(AttestationError::Invalid))
        }
    };

    match result {
        Ok(proof) => {
            let onchain_request = state.onchain_request_repo.find(request_id).await;
            match onchain_request {
                Ok(onchain_request) => {
                    tracing::info!("Onchain request found for request ID: {}", request_id);
                    tracing::info!("Onchain request: {:?}", onchain_request);
                    let result: Result<(bool, Vec<u8>, Option<prove::SubmitProofResponse>), anyhow::Error> = prove::submit_proof(onchain_request, proof.proof).await;
                    match result {
                        Ok((chain_verified, chain_raw_verified_output, response)) => {
                            tracing::info!("Proof submitted for request ID: {} chain_verified: {} chain_raw_verified_output: {}",
                                request_id, chain_verified, hex::encode(&chain_raw_verified_output));
                            if response.is_some() {
                                tracing::info!("Submit proof response: {:#?}", response);
                                let response = response.unwrap();
                                tracing::info!("Transaction hash: {}", hex::encode(&response.transaction_hash));

                                // Update onchain request status
                                match state.quote_repo.update_status(
                                    quote_id,
                                    response.proof_type,
                                    response.status,
                                    Some(response.transaction_hash),
                                    None,
                                ).await {
                                    Ok(_) => tracing::info!("tdx_quote updated successfully {quote_id} {}", response.status),
                                    Err(e) => tracing::error!("Failed to update quote status on success: {}", e)
                                }
                            }
                            Ok(())
                        }
                        Err(e) => {
                            tracing::error!("Failed to submit proof for request ID: {} {}", request_id, e);
                            match state.quote_repo.update_status(
                                quote_id,
                                ProofType::Sp1,
                                TdxQuoteStatus::Failure,
                                None,
                                None,
                            ).await {
                                Ok(_) => tracing::info!("tdx_quote updated successfully {quote_id} {}", TdxQuoteStatus::Failure),
                                Err(e) => tracing::error!("Failed to update quote status on failure: {}", e)
                            };
                            Err(e.into())
                        }
                    }
                }
                _ => {
                    tracing::error!("Failed to fetch onchain request for request ID: {}", request_id);
                    Err(Box::new(AttestationError::Invalid))
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to generate proof for request ID: {}", request_id);
            Err(e)
        }
    }
}
