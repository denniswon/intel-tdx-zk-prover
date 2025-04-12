use crate::handler::request_handler;
use crate::state::request_state::RequestState;
use axum::{
    routing::{delete, get, post},
    Router,
};

pub fn routes() -> Router<RequestState> {
    Router::new()
        .route("/request/register", post(request_handler::register))
        .route("/request/{id}", get(request_handler::query))
        .route("/request/{id}", delete(request_handler::delete))
}
