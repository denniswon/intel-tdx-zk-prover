use std::sync::Arc;

use dotenvy::dotenv;
use tracing::{Level, info};

use crate::config::database::{Database, DatabaseTrait};
use crate::config::parameter;

mod config;
mod dto;
mod entity;
mod error;
mod handler;
mod middleware;
mod repository;
mod response;
mod routes;
mod service;
mod state;
mod sp1;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // initialize tracing for logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    parameter::init();
    let connection = Database::init()
        .await
        .unwrap_or_else(|e| panic!("Database error: {}", e));

    let port = std::env::var("PORT")
        .or_else(|_| Ok::<String, std::env::VarError>("5000".to_string()))
        .unwrap();

    let host = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&host).await.unwrap();
    axum::serve(listener, routes::root::routes(Arc::new(connection)))
        .await
        .unwrap_or_else(|e| panic!("Server error: {}", e));

    info!("Server is running on {}", host);
}
