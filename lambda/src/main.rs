use lambda_runtime::{service_fn, Error};

mod function;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = rustls::crypto::ring::default_provider().install_default();

    tracing_subscriber::fmt().json()
        .with_max_level(tracing::Level::INFO)
        // this needs to be set to remove duplicated information in the log.
        .with_current_span(false)
        // this needs to be set to false, otherwise ANSI color codes will
        // show up in a confusing manner in CloudWatch logs.
        .with_ansi(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        // remove the name of the function from every log entry
        .with_target(false)
        .init();

    lambda_runtime::run(service_fn(move |event| async move {
        function::handler(event).await
    }))
    .await
}
