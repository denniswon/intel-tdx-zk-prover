use crate::handler::request_handler;
use crate::state::request_state::RequestState;
use axum::{
    Router,
    routing::{get, post},
};

pub fn routes() -> Router<RequestState> {
    Router::new()
        .route("/request/register", post(request_handler::register))
        .route("/request/{id}", get(request_handler::query))
}
