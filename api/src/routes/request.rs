use prover::state::request_state::RequestState;
use axum::{routing::get, Router};

use crate::handler::request_handler;

pub fn routes() -> Router<RequestState> {
    Router::new()
        .route("/request/{id}", get(request_handler::query))
}
