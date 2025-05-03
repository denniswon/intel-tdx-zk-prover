use std::fs::File;
use std::io::{BufReader, BufRead};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_eventbridge::types::PutEventsRequestEntry;
use aws_sdk_eventbridge::Client;
use lambda_runtime::Error;
use rand::Rng;

use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};
use prover::entity::{quote::{ProofType, TdxQuoteStatus}, zk::ProofSystem};
use prover::state::request_state::RequestState;
use prover::config::database::{Database, DatabaseTrait};
use prover::repository::request_repository::{OnchainRequestRepositoryTrait, OnchainRequestId};
use hex::FromHex;
use prover::config::parameter;

mod prove;

#[derive(Parser)]
#[command(name = "TDXProver")]
#[command(version = "0.1.0")]
#[command(about = r#"
|-----------------------------------------------------------------------------|
|      _______  _______  __   __     _______  _______  _______  ______        |
|     |       ||       ||  | |  |   |       ||       ||       ||    _ |       |
|     |    ___||   _   ||  |_|  |   |    ___||    ___||    ___||   | ||       |
|     |   |___ |  | |  ||       |   |   |___ |   |___ |   |___ |   |_||_      |
|     |    ___||  |_|  ||       |   |    ___||    ___||    ___||    __  |     |
|     |   |    |       | |     |    |   |    |   |    |   |___ |   |  | |     |
|     |___|    |_______|  |___|     |___|    |___|    |_______||___|  |_|     |
|                                                                             |
|           TDX PROVER CLI — Trustless Proofs, Verified Execution             |
|                                                                             |
|       Fetch requests, generate sp1 or risc0 proofs, & submit on-chain.      |
|-----------------------------------------------------------------------------|
"#)]
#[command(after_help = r#"
                       ╔═══════════════════════════════╗
                       ║        PROOF COMPLETE         ║
                       ║   Trust, but verify — TDX+ZK  ║
                       ╚═══════════════════════════════╝
"#)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Proves a TDX quote and submits it on-chain
    Prove(ProveArgs),

    /// Load tests the prover flow
    LoadTest(LoadTestArgs),

    /// Load tests the remote prover flow in lambda
    LoadTestLambda(LoadTestLambdaArgs),
}

/// Enum representing the available proof types
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ProofTypeArg {
    Sp1,
    Risc0,
}

/// Enum representing the available proof systems
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ProofSystemArg {
    Groth16,
    Plonk,
}

/// Enum representing the available quote statuses
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum TdxQuoteStatusArg {
    Pending,
    Failure,
    Success,
}

#[derive(Args, Debug)]
struct ProveArgs {
    /// The onchain_request_id string to prove
    #[arg(short = 'i', long = "onchain-request-id")]
    request_id: String,

    #[arg(
        short = 't',
        long = "proof-type",
        value_enum,
        default_value = "sp1"
    )]
    proof_type: Option<ProofTypeArg>,

    #[arg(
        short = 'z',
        long = "proof-system",
        value_enum,
        default_value = "groth16"
    )]
    proof_system: Option<ProofSystemArg>,

    #[arg(
        short = 'v',
        long = "verify-only",
        default_value = "false",
        help = "If true, make static call to automata testnet contract"
    )]
    verify_only: Option<bool>,
}

#[derive(Args, Debug)]
struct LoadTestArgs {
    #[arg(short = 'f', long = "file")]
    input_file: Option<PathBuf>,

    #[arg(
        short = 'c',
        long = "count",
        default_value = "10",
        help = "Number of requests from db to load test if input_file is not specified"
    )]
    count: Option<u64>,

    #[arg(short = 'd', long = "delay-milliseconds", default_value = "1000")]
    delay_milliseconds: Option<u64>,

    #[arg(
        short = 't',
        long = "proof-type",
        value_enum,
        help = "If not specified, chosen randomly"
    )]
    proof_type: Option<ProofTypeArg>,

    #[arg(
        short = 'z',
        long = "proof-system",
        value_enum,
        default_value = "groth16"
    )]
    proof_system: Option<ProofSystemArg>,

    #[arg(
        short = 'v',
        long = "verify-only",
        default_value = "true",
        help = "If true, skip submitting proof on mainnet"
    )]
    verify_only: Option<bool>,

    #[arg(
        short = 's',
        long = "quote-status",
        value_enum,
        default_value = "pending"
    )]
    quote_status: Option<TdxQuoteStatusArg>,
}

#[derive(Args, Debug)]
struct LoadTestLambdaArgs {
    #[arg(
        short = 'c',
        long = "count",
        default_value = "10",
        help = "Number of requests from db to load test if input_file is not specified"
    )]
    count: Option<u64>,

    #[arg(short = 'd', long = "delay-milliseconds", default_value = "1000")]
    delay_milliseconds: Option<u64>,

    #[arg(
        short = 't',
        long = "proof-type",
        value_enum,
        help = "If not specified, chosen randomly"
    )]
    proof_type: Option<ProofTypeArg>,

    #[arg(
        short = 's',
        long = "quote-status",
        value_enum,
        default_value = "pending"
    )]
    quote_status: Option<TdxQuoteStatusArg>,
}

#[tokio::main]
async fn main() -> Result<()> {
    parameter::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Prove(args) => {
            let request_id = Vec::from_hex(
                args.request_id.strip_prefix("0x").unwrap_or(args.request_id.as_str()))
                    .unwrap_or_else(|e| panic!("Invalid hex string: {}", e)
            );

            let proof_type = match args.proof_type.unwrap_or(ProofTypeArg::Sp1) {
                ProofTypeArg::Sp1 => ProofType::Sp1,
                ProofTypeArg::Risc0 => ProofType::Risc0,
            };
            let proof_system = match args.proof_system.unwrap_or(ProofSystemArg::Groth16) {
                ProofSystemArg::Groth16 => ProofSystem::Groth16,
                ProofSystemArg::Plonk => ProofSystem::Plonk,
            };
            let verify_only = args.verify_only.unwrap_or(false);

            println!("Proving request_id: {} with proof_type: {} and proof_system: {} (verify_only: {})",
                hex::encode(&request_id), proof_type, proof_system, verify_only);

            prove::handler(request_id, proof_type, proof_system, verify_only).await
        }
        Commands::LoadTest(args) => {
            let count = args.count.unwrap_or(10);

            let delay_milliseconds = args.delay_milliseconds.unwrap_or(1000);

            let proof_system = match args.proof_system.unwrap_or(ProofSystemArg::Groth16) {
                ProofSystemArg::Groth16 => ProofSystem::Groth16,
                ProofSystemArg::Plonk => ProofSystem::Plonk,
            };

            let verify_only = args.verify_only.unwrap_or(true);

            let quote_status = match args.quote_status.unwrap_or(TdxQuoteStatusArg::Pending) {
                TdxQuoteStatusArg::Pending => TdxQuoteStatus::Pending,
                TdxQuoteStatusArg::Failure => TdxQuoteStatus::Failure,
                TdxQuoteStatusArg::Success => TdxQuoteStatus::Success,
            };

            println!("Load testing (input_file: {:?}, count: {}, delay_milliseconds: {}, quote_status: {}, verify_only: {})",
                &args.input_file, count, delay_milliseconds, quote_status, verify_only);
            
            let onchain_request_ids = match &args.input_file {
                Some(input_file) => read_lines(input_file).unwrap(),
                None => fetch_onchain_request_ids(Some(quote_status), count).await,
            };

            for request_id in onchain_request_ids {
                println!("Proving onchain_request: {}", hex::encode(&request_id.request_id));

                let proof_type = match args.proof_type {
                    Some(proof_type) => match proof_type {
                        ProofTypeArg::Sp1 => ProofType::Sp1,
                        ProofTypeArg::Risc0 => ProofType::Risc0,
                    },
                    None => {
                        let mut rng = rand::rng();
                        let random_bool: bool = rng.random();
                        if random_bool {
                            ProofType::Sp1
                        } else {
                            ProofType::Risc0
                        }
                    }
                };
                
                let _ = prove::handler(request_id.request_id, proof_type, proof_system, verify_only).await;
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_milliseconds)).await;
            }
            println!("Finished load testing");
            Ok(())
        }
        Commands::LoadTestLambda(args) => {
            let count = args.count.unwrap_or(10);

            let delay_milliseconds = args.delay_milliseconds.unwrap_or(1000);

            let quote_status = match args.quote_status.unwrap_or(TdxQuoteStatusArg::Pending) {
                TdxQuoteStatusArg::Pending => TdxQuoteStatus::Pending,
                TdxQuoteStatusArg::Failure => TdxQuoteStatus::Failure,
                TdxQuoteStatusArg::Success => TdxQuoteStatus::Success,
            };

            println!("Load testing tdx-prover lambda function (count: {}, delay_milliseconds: {}, quote_status: {})",
                count, delay_milliseconds, quote_status);
            
            let onchain_request_ids = fetch_onchain_request_ids(Some(quote_status), count).await;

            for request_id in onchain_request_ids {
                println!("Requesting tdx-prover lambda for request: {}", hex::encode(&request_id.request_id));

                let proof_type = match args.proof_type {
                    Some(proof_type) => match proof_type {
                        ProofTypeArg::Sp1 => ProofType::Sp1,
                        ProofTypeArg::Risc0 => ProofType::Risc0,
                    },
                    None => {
                        let mut rng = rand::rng();
                        let random_bool: bool = rng.random();
                        if random_bool {
                            ProofType::Sp1
                        } else {
                            ProofType::Risc0
                        }
                    }
                };

                let region_provider = RegionProviderChain::default_provider().or_else("us-west-2");
                let config = aws_config::defaults(BehaviorVersion::latest())
                    .region(region_provider).load().await;
                let client = Arc::new(Client::new(&config));

                let _ = request_tdx_prover_lambda(&client, request_id.request_id, proof_type).await;
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_milliseconds)).await;
            }
            println!("Finished load testing tdx-prover lambda");
            Ok(())
        }
    }
}

fn read_lines<P>(filename: P) -> Result<Vec<OnchainRequestId>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    let buf = BufReader::new(file);
    let onchain_request_ids = buf.lines().filter_map(Result::ok)
        .map(|line| {
            Vec::from_hex(
                line.strip_prefix("0x").unwrap_or(line.as_str())
            ).unwrap_or_else(|_e| vec![])
        }).collect::<Vec<Vec<u8>>>()
        .into_iter().filter(|x| x.len() > 0).collect::<Vec<Vec<u8>>>()
        .into_iter().map(|request_id| {
            OnchainRequestId::new(request_id)
        }).collect::<Vec<OnchainRequestId>>();
    Ok(onchain_request_ids)
}

async fn fetch_onchain_request_ids(status: Option<TdxQuoteStatus>, count: u64) -> Vec<OnchainRequestId> {
    let db_conn = Arc::new(
        Database::init()
            .await
            .unwrap_or_else(|e| panic!("Database error: {}", e)),
    );
    let request_state = RequestState::new(&db_conn);

    request_state.request_repo
        .find_request_ids_by_status(status, Some(count as i64))
        .await    
}

async fn request_tdx_prover_lambda(client: &Arc<Client>, request_id: Vec<u8>, proof_type: ProofType) -> Result<(), Error> {
    let source = "com.magic.newton";
    let detail_type = "tdx-prover";
    let detail = format!(r#"{{"request_id": "{}", "proof_type": "{}"}}"#, hex::encode(&request_id), proof_type);
    let event_bus_name = "tdx-prover-bus";
    
    client
        .put_events()
        .entries(
            PutEventsRequestEntry::builder()
                .event_bus_name(event_bus_name)
                .source(source)
                .detail_type(detail_type)
                .detail(detail)
                .build()
        )
        .send()
        .await
        .map_err(|e| {
            println!("Failed to request tdx-prover lambda: {}", e);
            Error::from(e)
        })?;
    
    println!("Successfully requested tdx-prover lambda for request: {}", hex::encode(&request_id));
    Ok(())
}