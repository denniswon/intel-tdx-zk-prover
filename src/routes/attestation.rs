use crate::handler::attestation_handler;
use axum::{routing::{get, post}, Router};

pub fn routes() -> Router {
    let router = Router::new()
        .route("/attestation/register", post(attestation_handler::create))
        .route("/attestation/{id}", get(attestation_handler::query));
    return router;
}
