use crate::handler::register_handler;
use crate::state::model_state::ModelState;
use axum::{Router, routing::post};

pub fn model() -> Router<ModelState> {
    let router = Router::new().route("/register", post(register_handler::register));
    return router;
}
