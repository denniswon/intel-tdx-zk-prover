use std::sync::Arc;

use aws_config::BehaviorVersion;
use aws_sdk_eventbridge as eventbridge;

use dotenvy::dotenv;
use error::api_error::ApiError;
use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::{json, Value};
use tracing::{Level, info};

use crate::config::database::{Database, DatabaseTrait};
use crate::config::parameter;
use crate::eventbridge::types::PutEventsRequestEntry;

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

#[::tokio::main]
async fn main() -> Result<(), Error> {
    let lambda = std::env::var("LAMBDA")
        .or_else(|_| Ok::<String, std::env::VarError>("false".to_string()))
        .unwrap();

    match lambda.as_str() {
        "true" => lambda_runtime::run(service_fn(handler)).await,
        _ => {
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
            Ok(())
        }
    }
}

async fn handler(_event: LambdaEvent<Value>) -> Result<Value,Error> {
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;

    let client = aws_sdk_eventbridge::Client::new(&config);

    // TODO: Get request_id from lambda _event above
    let request_id = _event.payload["detail"]["request_id"].as_str().unwrap();
    let event = format!(r#"
    {{
        "request_id": "{request_id}"
    }}"#);

    let input = PutEventsRequestEntry::builder()
        .detail(event)
        .detail_type("tdx-prover-type".to_string())
        .event_bus_name("tdx-prover-bus".to_string())
        .source("tdx-prover-source".to_string())
        .build();

    match client.put_events().entries(input).send().await {
        Ok(result) => {
            println!("Event sent: {}", result.failed_entry_count);
            Ok(json!({ "message": format!("Request {request_id} event sent") }))
        },
        Err(err) => match err.into_service_error() {
            eventbridge::operation::put_events::PutEventsError::InternalException(e) => {
                 println!("eventbridge error: {:?}", &e.message().unwrap());
                 Ok(json!({ "Event Error": format!("Failed to send Request {request_id} event") }))
            },
            e => {
                Err(ApiError::EventBridgeError(e))?
            }
        },
    }
}
