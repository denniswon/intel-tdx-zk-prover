use std::{env, sync::Arc, time::Duration, time::Instant};

use alloy::transports::http::reqwest;
use aws_config::BehaviorVersion;
use aws_lambda_events::eventbridge::EventBridgeEvent;
use aws_sdk_eventbridge as eventbridge;

use dotenvy::dotenv;
use error::api_error::ApiError;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{json, Value};
use tracing::{info, Level};

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
mod sp1;
mod state;

struct HandlerConfig {
    url: reqwest::Url,
    client: reqwest::Client,
}

async fn handler(
    _config: &HandlerConfig,
    event: LambdaEvent<EventBridgeEvent<Value>>,
) -> Result<Value, Error> {
    let start = Instant::now();
    let _duration = start.elapsed();

    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;

    let client = aws_sdk_eventbridge::Client::new(&config);

    let request_id = event.payload.detail["request_id"].as_str().unwrap();
    let event = format!(
        r#"
    {{
        "request_id": "{request_id}"
    }}"#
    );

    println!("Event: {}", event);

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
        }
        Err(err) => match err.into_service_error() {
            eventbridge::operation::put_events::PutEventsError::InternalException(e) => {
                println!("eventbridge error: {:?}", &e.message().unwrap());
                Ok(json!({ "Event Error": format!("Failed to send Request {request_id} event") }))
            }
            e => Err(ApiError::PutEventsError(e))?,
        },
    }
}

#[::tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    parameter::init();

    // initialize tracing for logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let lambda = std::env::var("LAMBDA")
        .or_else(|_| Ok::<String, std::env::VarError>("false".to_string()))
        .unwrap();

    match lambda.as_str() {
        "true" => {
            let timeout = env::var("TIMEOUT").unwrap_or_else(|_| "60".to_string());
            let timeout = timeout
                .parse::<u64>()
                .expect("TIMEOUT environment variable is not a valid number");
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(timeout))
                .build()?;

            let config = &HandlerConfig {
                url: reqwest::Url::parse(&parameter::get("DATABASE_URL")).unwrap(),
                client: reqwest::Client::new(),
            };
            lambda_runtime::run(service_fn(move |event| async move {
                handler(config, event).await
            }))
            .await
        }
        _ => {
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
