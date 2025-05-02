use lambda_runtime::{service_fn, Error};

mod function;

#[::tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::run(service_fn(move |event| async move {
        function::handler(event).await
    }))
    .await
}
