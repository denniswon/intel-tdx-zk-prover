use crate::handler::request_handler;
use crate::state::request_state::RequestState;
use axum::{routing::get, Router};

pub fn routes() -> Router<RequestState> {
    Router::new()
        .route("/request/{id}", get(request_handler::query))
}
