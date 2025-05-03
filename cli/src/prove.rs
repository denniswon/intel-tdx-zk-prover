use std::sync::Arc;
use anyhow::Error;
use prover::{
    config::database::{Database, DatabaseTrait},
    entity::quote::{ProofType, TdxQuoteStatus},
    entity::zk::ProofSystem,
    error::{db_error::DbError, quote_error::QuoteError},
    repository::{quote_repository::QuoteRepositoryTrait, request_repository::OnchainRequestRepositoryTrait},
    state::{quote_state::QuoteState, request_state::RequestState}, zk,
};

pub(crate) async fn handler(
    request_id: Vec<u8>,
    proof_type: ProofType,
    proof_system: ProofSystem,
    verify_only: bool
) -> Result<(), Error> {
    let request_id_hex = hex::encode(&request_id);

    let db_conn = Arc::new(
        Database::init()
            .await
            .unwrap_or_else(|e| panic!("Database error: {}", e)),
    );

    let quote_state = QuoteState::new(&db_conn);
    let request_state = RequestState::new(&db_conn);

    let onchain_request = request_state.request_repo.find_by_request_id(request_id).await.map_err(|e| {
        println!("Failed to fetch onchain request: {}", e);
        DbError::SomethingWentWrong("Failed to fetch onchain request".to_string())
    })?;

    println!("Onchain request found: {:#?}", onchain_request);

    let attestation = quote_state.quote_repo.find_by_onchain_request_id(onchain_request.id).await.map_err(|e| {
        println!("Failed to fetch attestation: {}", e);
        DbError::SomethingWentWrong("Failed to fetch attestation".to_string())
    })?;

    let quote_id = attestation.id;
    println!("Attestation found for request ID: {} {}", request_id_hex, attestation.status);
    let proof = zk::prove(attestation.quote, proof_type, Some(proof_system)).await
        .map_err(|e| {
            println!("Failed to generate proof for request ID: {:?} {}", request_id_hex, e.to_string());
            QuoteError::Prove
        })?;
    
    println!("Proof generated for request ID: {:?} {:#?}", request_id_hex, proof.proof);

    // only verify proof in dev because in lambda, filesystem is not writable
    if std::env::var("ENV").unwrap_or("dev".to_string()) != "prod" {
        println!("Verifying proof...");
        
        zk::verify_proof(&proof.proof).await.map_err(|e| {
            println!("Failed to verify proof: {}", e);
            QuoteError::VerifyProof
        })?;
        println!("Successfully verified proof.");
    }

    let (verified, raw_verified_output, tx_hash, response) =
        zk::submit_proof(onchain_request, proof_type, proof.proof, Some(verify_only)).await
            .map_err(|e| {
                println!("Failed to submit proof: {}", e);
                QuoteError::SubmitProof
            })?;

    println!(
        "Proof submitted for request ID: {} verified: {} raw_verified_output: {}",
        request_id_hex, verified, hex::encode(&raw_verified_output)
    );

    if response.is_some() {
        println!("Submit proof response: {:#?}", response);
        let response = response.unwrap();
        println!("Transaction hash: {}", hex::encode(&response.transaction_hash));

        // Update onchain request status
        quote_state.quote_repo.update_status(
            quote_id,
            proof_type,
            TdxQuoteStatus::Failure,
            Some(response.transaction_hash.to_vec()),
            None,
        ).await.map_err(|e| {
            println!("Failed to update quote status on success: {}", e);
            QuoteError::UpdateStatusOnSuccess
        })?;

        println!("tdx_quote updated successfully {quote_id} {}", TdxQuoteStatus::Success);
    } else if tx_hash.is_some() {
        println!("Transaction hash: {}", hex::encode(&tx_hash.unwrap().to_vec()));
        // Update onchain request status
        quote_state.quote_repo.update_status(
            quote_id,
            proof_type,
            TdxQuoteStatus::Failure,
            Some(tx_hash.unwrap().to_vec()),
            None,
        ).await.map_err(|e| {
            println!("Failed to update quote status on failure: {}", e);
            QuoteError::UpdateStatusOnFailure
        })?;

        println!("tdx_quote updated successfully {quote_id} {}", TdxQuoteStatus::Failure);
    }

    Ok(())
}
