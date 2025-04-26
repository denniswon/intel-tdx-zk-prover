use std::sync::Arc;

use crate::{
    config::{
        database::{Database, DatabaseTrait},
        parameter,
    },
    entity::quote::{ProofType, TdxQuoteStatus},
    error::{db_error::DbError, quote_error::QuoteError},
    repository::{quote_repository::QuoteRepositoryTrait, request_repository::OnchainRequestRepositoryTrait},
    sp1::prove,
    state::{quote_state::QuoteState, request_state::RequestState}
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
    let request_id_hex = request_id.as_str().unwrap();
    let request_id = hex::decode(request_id_hex).unwrap();

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

    let attestation = quote_state.quote_repo.find_by_onchain_request_id(onchain_request.id).await;
    let mut quote_id: Uuid = Uuid::nil();
    let result = match attestation {
        Ok(attestation) => {
            quote_id = attestation.id;
            tracing::info!("Attestation found for request ID: {:?} {}", request_id_hex, attestation.status);
            let proof = prove::prove(attestation.quote.clone(), None).await;
            match proof {
                Ok(proof) => {
                    tracing::info!("Proof generated for request ID: {:?}", request_id_hex);
                    Ok(proof)
                }
                Err(e) => {
                    tracing::error!("Failed to generate proof for request ID: {:?} {}", request_id_hex, e.to_string());
                    Err(Box::new(QuoteError::Invalid))
                }
            }
        }
        _ => {
            tracing::error!("Attestation not found for request ID: {}", request_id_hex);
            Err(Box::new(QuoteError::Invalid))
        }
    };

    match result {
        Ok(proof) => {
            tracing::info!("Verifying proof...");
            let _ = prove::verify_proof(proof.proof.clone()).await;

            match prove::submit_proof(onchain_request, proof.proof).await {
                Ok((verified, verified_output, tx_hash, response)) => {
                    tracing::info!("Proof submitted for request ID: {} verified: {} verified_output: {}",
                        request_id_hex, verified, hex::encode(&verified_output));
                    
                    if response.is_some() {
                        tracing::info!("Submit proof response: {:#?}", response);
                        let response = response.unwrap();
                        tracing::info!("Transaction hash: {}", hex::encode(&response.transaction_hash));

                        // Update onchain request status
                        match quote_state.quote_repo.update_status(
                            quote_id,
                            response.proof_type,
                            response.status,
                            Some(tx_hash.unwrap().to_vec()),
                            None,
                        ).await {
                            Ok(_) => tracing::info!("tdx_quote updated successfully {quote_id} {}", response.status),
                            Err(e) => tracing::error!("Failed to update quote status on success: {}", e)
                        }
                    } else if tx_hash.is_some() {
                        tracing::info!("Transaction hash: {}", hex::encode(&tx_hash.unwrap()));
                        // Update onchain request status
                        match quote_state.quote_repo.update_status(
                            quote_id,
                            ProofType::Sp1,
                            TdxQuoteStatus::Failure,
                            Some(tx_hash.unwrap().to_vec()),
                            None,
                        ).await {
                            Ok(_) => tracing::info!("tdx_quote updated successfully {quote_id} {}", TdxQuoteStatus::Failure),
                            Err(e) => tracing::error!("Failed to update quote status on failure: {}", e)
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    tracing::error!("Failed to submit proof for request ID: {} {}", request_id_hex, e);
                    match quote_state.quote_repo.update_status(
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
        Err(e) => {
            tracing::error!("Failed to generate proof for request ID: {} {}", request_id_hex, e);
            Err(e.into())
        }
    }
}
