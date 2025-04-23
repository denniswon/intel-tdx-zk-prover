use std::sync::Arc;

use lambda_runtime::{service_fn, Error};
use tracing::{info, Level};

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
mod sp1;
mod state;
mod lambda;

#[::tokio::main]
async fn main() -> Result<(), Error> {
    let lambda = std::env::var("LAMBDA")
        .or_else(|_| Ok::<String, std::env::VarError>("false".to_string()))
        .unwrap();

    match lambda.as_str() {
        "true" => {
            lambda_runtime::run(service_fn(move |event| async move {
                lambda::function::handler(event).await
            }))
            .await
        }
        _ => {
            parameter::init();

            // initialize tracing for logging
            tracing_subscriber::fmt().with_max_level(Level::INFO).init();

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
            Ok(())
        }
    }
}
