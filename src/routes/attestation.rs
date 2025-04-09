use crate::handler::attestation_handler;
use crate::state::attestation_state::AttestationState;
use axum::{routing::{get, post}, Router};

pub fn routes() -> Router<AttestationState> {
    
    Router::new()
        .route("/attestation/register", post(attestation_handler::register))
        .route("/attestation/{id}", get(attestation_handler::query))
}
