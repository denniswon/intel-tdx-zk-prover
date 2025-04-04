use std::sync::Arc;
use crate::config::{database, parameter};
use crate::config::database::DatabaseTrait;

use tracing::{info, Level};
use tracing_subscriber;

mod config;
mod routes;
mod dto;
mod error;
mod response;
mod entity;
mod repository;
mod state;
mod service;
mod middleware;
mod handler;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // initialize tracing for logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    parameter::init();
    let connection = database::Database::init()
        .await
        .unwrap_or_else(|e| panic!("Database error: {}", e.to_string()));

    let port = std::env::var("PORT").or_else(|_| Ok("5000".to_string()));

    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .layer(Extension(pool));

    let host = format!("0.0.0.0:{}", port);
    axum::Server::bind(&host.parse().unwrap())
        .serve(routes::root::routes(Arc::new(connection)))
        .await
        .unwrap_or_else(|e| panic!("Server error: {}", e.to_string()));
    
    info!("Server is running on {}", host);
}
