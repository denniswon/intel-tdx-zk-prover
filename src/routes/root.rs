use super::request;
use crate::config::database::Database;
use crate::middleware::auth as auth_middleware;
use crate::routes::{attestation, register};
use crate::state::auth_state::AuthState;
use crate::state::request_state::UserState;
use crate::state::token_state::TokenState;
use axum::routing::{IntoMakeService, get};
use axum::{Router, middleware};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

pub fn routes(db_conn: Arc<Database>) -> IntoMakeService<Router> {
    let merged_router = {
        let agent_state = AgentState::new(&db_conn);
        let attestation_state = AttestationState::new(&db_conn);
        let request_state = RequestState::new(&db_conn);

        request::routes()
            .with_state(request_state)
            .merge(attestation::routes().with_state(attestation_state))
            .merge(agent::routes().with_state(agent_state))
            .merge(Router::new().route("/health", get(|| async { "Healthy..." })))
    };

    let app_router = Router::new()
        .nest("/api", merged_router)
        .layer(TraceLayer::new_for_http());

    app_router.into_make_service()
}
