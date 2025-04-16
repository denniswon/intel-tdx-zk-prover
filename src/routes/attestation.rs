use crate::handler::attestation_handler;
use crate::state::attestation_state::AttestationState;
use axum::{routing::{get, post}, Router};

pub fn routes() -> Router<AttestationState> {
    
    Router::new()
        .route("/attestation/register", post(attestation_handler::register))
        .route("/attestation/{id}", get(attestation_handler::query))
        .route("/attestation/verify_dcap_qvl/{id}", get(attestation_handler::verify_dcap_qvl))
        .route("/attestation/verify_dcap/{id}", get(attestation_handler::verify_dcap))
}
