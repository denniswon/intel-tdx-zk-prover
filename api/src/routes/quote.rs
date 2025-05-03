use tdx_prover::state::quote_state::QuoteState;
use axum::{routing::{get, post}, Router};

use crate::handler::quote_handler;

pub fn routes() -> Router<QuoteState> {
    
    Router::new()
        .route("/quote/register", post(quote_handler::register))
        .route("/quote/{id}", get(quote_handler::query))
        .route("/quote/verify_dcap/{id}", get(quote_handler::verify_dcap))
        .route("/quote/prove/{id}", get(quote_handler::prove))
        .route("/quote/verify", post(quote_handler::verify))
        .route("/quote/submit_proof", post(quote_handler::submit_proof))
}
