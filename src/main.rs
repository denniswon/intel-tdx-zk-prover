use std::sync::Arc;
use axum::{Router, Extension, routing::get};
use dotenvy::dotenv;
use routes::root;
use tracing::{info, Level};
use tracing_subscriber;

use crate::config::parameter;
use crate::config::database::{Database, DatabaseTrait};

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
    let connection = Database::init()
        .await
        .unwrap_or_else(|e| panic!("Database error: {}", e.to_string()));

    let port = std::env::var("PORT")
        .or_else(|_| Ok::<String, std::env::VarError>("5000".to_string())).unwrap();

    // build our application with a route
    let app = Router::new()
        .route("/", get(get(root)))
        .layer(Extension(connection));

    let host = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&host).await.unwrap();
    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| panic!("Server error: {}", e.to_string()));
    
    info!("Server is running on {}", host);
}

async fn root() -> &'static str {
    "Hello, world!"
}