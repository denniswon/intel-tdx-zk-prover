use tdx_prover::config::database::Database;
use tdx_prover::state::quote_state::QuoteState;
use tdx_prover::state::request_state::RequestState;
use axum::body::Bytes;
use axum::routing::{IntoMakeService, get};
use axum::Router;
use tower_http::classify::ServerErrorsFailureClass;
use std::sync::Arc;
use std::time::Duration;
use tower_http::LatencyUnit;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};

use super::{quote, request};

pub fn routes(db_conn: Arc<Database>) -> IntoMakeService<Router> {
    let merged_router = {
        let quote_state = QuoteState::new(&db_conn);
        let request_state = RequestState::new(&db_conn);

        request::routes()
            .with_state(request_state)
            .merge(quote::routes().with_state(quote_state))
            .merge(Router::new().route("/health", get(|| async { "Healthy..." })))
    };

    let app_router = Router::new()
        .nest("/api", merged_router)
        .layer(
            TraceLayer::new_for_http()
                .on_body_chunk(|chunk: &Bytes, latency: Duration, _: &tracing::Span| {
                    tracing::trace!(size_bytes = chunk.len(), latency = ?latency, "sending body chunk")
                })
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_response(DefaultOnResponse::new().include_headers(true).latency_unit(LatencyUnit::Micros))
                .on_failure(
                    |error: ServerErrorsFailureClass, latency: Duration, _span: &tracing::Span| {
                        tracing::error!("Request failed: {:?}", error);
                        tracing::debug!("Request failed after {:?}", latency);
                    },
                ),
        );

    app_router.into_make_service()
}
