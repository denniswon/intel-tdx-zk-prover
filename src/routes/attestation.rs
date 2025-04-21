use crate::handler::attestation_handler;
use crate::state::attestation_state::AttestationState;
use axum::{routing::{get, post}, Router};

pub fn routes() -> Router<AttestationState> {
    
    Router::new()
        .route("/attestation/register", post(attestation_handler::register))
        .route("/attestation/{id}", get(attestation_handler::query))
        .route("/attestation/verify_dcap/{id}", get(attestation_handler::verify_dcap))
        .route("/attestation/prove/{id}", get(attestation_handler::prove))
        .route("/attestation/verify", post(attestation_handler::verify))
        .route("/attestation/submit_proof", post(attestation_handler::submit_proof))
}
