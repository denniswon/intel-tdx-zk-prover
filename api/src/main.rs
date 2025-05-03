use std::sync::Arc;
use tracing::info;
use tdx_prover::config::database::{Database, DatabaseTrait};
use tdx_prover::config::parameter;

mod error;
mod handler;
mod middleware;
mod response;
mod routes;

#[::tokio::main]
async fn main() {
    parameter::init();

    // initialize tracing for logging
    tracing_subscriber::fmt().init();

    let connection = Database::init()
        .await
        .unwrap_or_else(|e| panic!("Database error: {}", e));

    let port = std::env::var("PORT")
        .or_else(|_| Ok::<String, std::env::VarError>("8002".to_string()))
        .unwrap();

    let host = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&host).await.unwrap();
    axum::serve(listener, routes::root::routes(Arc::new(connection)))
        .await
        .unwrap_or_else(|e| panic!("Server error: {}", e));

    info!("Server is running on {}", host);
}
