use std::sync::Arc;

use aws_lambda_events::eventbridge::EventBridgeEvent;
use lambda_runtime::{LambdaEvent, Error};
use crate::{
    config::{database::{Database, DatabaseTrait}, parameter}, error::attestation_error::AttestationError, repository::attestation_repository::AttestationRepositoryTrait, sp1::prove::{self, submit_proof, DcapProof}, state::attestation_state::AttestationState
};
use tracing::Level;

pub(crate)async fn handler(event: LambdaEvent<EventBridgeEvent>) -> Result<(), Error> {
    println!("Event: {:?}", event);
    parameter::init();
    // initialize tracing for logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // Extract some useful information from the request
    let payload = event.payload;
    tracing::info!("Payload: {:?}", payload);

    let request_id = payload.detail.get("request_id").unwrap();
    tracing::info!("Request ID: {}", request_id);
    let request_id = request_id.as_u64().unwrap_or_else(|| 1u64);

    let db_conn = Arc::new(Database::init()
        .await
        .unwrap_or_else(|e| panic!("Database error: {}", e)));

    let state = AttestationState::new(&db_conn);

    let attestation = state.attestation_repo.find(request_id).await;

    let result = match attestation {
        Ok(attestation) => {
            let proof = prove::prove(attestation.attestation_data, None).await;
            match proof {
                Ok(proof) => {
                    tracing::info!("Proof generated for request ID: {}", request_id);
                    Ok(proof)
                }
                _ => {
                    tracing::error!("Failed to generate proof for request ID: {}", request_id);
                    Err(Box::new(AttestationError::Invalid))
                }
            }
        },
        _ => {
            tracing::error!("Attestation not found for request ID: {}", request_id);
            Err(Box::new(AttestationError::Invalid))
        }
    };

    match result {
        Ok(proof) => {
            let result = submit_proof(proof).await;
            match result {
                Ok((chain_verified, chain_raw_verified_output)) => {
                    tracing::info!("Proof submitted for request ID: {} chain_verified: {} chain_raw_verified_output: {}",
                        request_id, chain_verified, hex::encode(&chain_raw_verified_output));
                    Ok(())
                }
                Err(e) => {
                    tracing::error!("Failed to submit proof for request ID: {}", request_id);
                    Err(e.into())
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to generate proof for request ID: {}", request_id);
            Err(e)
        }
    }
}