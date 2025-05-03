use std::fs::File;
use std::io::{BufReader, BufRead};
use std::path::{Path, PathBuf};
use std::sync::Arc;
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
#[command(about = "TDX Prover CLI to fetch onchain_requests from db, generate proofs using either sp1 or risc0, then submit on-chain")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Proves a TDX quote and submits it on-chain
    Prove(ProveArgs),

    /// Load tests the prover flow by putting events to tdx-prover event bus
    LoadTest(LoadTestArgs),
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
        short = 's',
        long = "proof-system",
        value_enum,
        default_value = "groth16"
    )]
    proof_system: Option<ProofSystemArg>,

    #[arg(
        short = 'v',
        long = "verify-only",
        default_value = "false",
        help = "If true, make staticcall to automata testnet contract instead"
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
        help = "If input_file is not specified, queries onchain_requests from db and triggers that many events"
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
        short = 'q',
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
                        let mut rng = rand::thread_rng();
                        let random_bool: bool = rng.gen();
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