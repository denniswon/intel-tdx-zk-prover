use aws_config::{meta::region::RegionProviderChain, BehaviorVersion, Region};
use aws_sdk_lambda::{
    operation::invoke::InvokeOutput,
    types::InvocationType,
    Client as LambdaClient
};
use aws_sdk_eventbridge::{
    operation::put_events::PutEventsOutput,
    types::PutEventsRequestEntry,
    Client as EventBridgeClient
};
use chrono::SecondsFormat;
use lambda_runtime::Error;
use tdx_prover::entity::quote::ProofType;
use std::sync::Arc;
use uuid::Uuid;
use serde_json;
use hex;

pub(crate) async fn init_aws() -> (Arc<LambdaClient>, Arc<EventBridgeClient>, Region) {
    let region_provider = RegionProviderChain::default_provider().or_else("us-west-2");
    let region = region_provider.region().await.unwrap();
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;
    let lambda_client = LambdaClient::new(&config);
    let event_bridge_client = EventBridgeClient::new(&config);
    println!("AWS clients initialized ({})", region.as_ref());
    (Arc::new(lambda_client), Arc::new(event_bridge_client), region)
}

pub(crate) async fn invoke_tdx_prover_lambda(
    client: &Arc<LambdaClient>,
    function_name: &str,
    region: &Region,
    request_id: Vec<u8>,
    proof_type: ProofType
) -> Result<InvokeOutput, Error> {
    println!("Invoking tdx-prover lambda for request: {}", hex::encode(&request_id));

    // Simulated EventBridge event payload
    let event_payload = serde_json::json!({
        "version": "0",
        "id": Uuid::new_v4().to_string(),
        "detail-type": "tdx-prover",
        "source": "com.magic.newton",
        "account": "637423541619",
        "time": chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        "region": region.as_ref(),
        "resources": [],
        "detail": {
        "request_id": hex::encode(&request_id),
        "proof_type": proof_type
        }
    });

    let payload_bytes = serde_json::to_vec(&event_payload)?;

    if std::env::var("DEBUG").is_ok() {
        println!("Lambda event payload: {:#?}", event_payload);
    }

    let resp = client
        .invoke()
        .function_name(function_name)
        .invocation_type(InvocationType::Event)
        .payload(payload_bytes.into())
        .send()
        .await?;

    Ok(resp)
}

pub(crate) async fn put_tdx_prover_event(
    client: &Arc<EventBridgeClient>,
    request_id: Vec<u8>,
    proof_type: ProofType
) -> Result<PutEventsOutput, Error> {
    println!("Putting event to tdx-prover event bus for request: {}", hex::encode(&request_id));

    let detail = serde_json::json!({
        "request_id": hex::encode(&request_id),
        "proof_type": proof_type
    });

    let resp = client
        .put_events()
        .entries(
            PutEventsRequestEntry::builder()
                .source("com.magic.newton")
                .detail_type("tdx-prover")
                .detail(detail.to_string())
                .event_bus_name("tdx-prover-bus")
                .build(),
        )
        .send()
        .await?;

    Ok(resp)
}
