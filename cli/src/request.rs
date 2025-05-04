use std::fs::File;
use std::io::{BufReader, BufRead};
use std::path::Path;
use std::sync::Arc;
use tdx_prover::entity::quote::TdxQuoteStatus;
use tdx_prover::state::request_state::RequestState;
use tdx_prover::config::database::{Database, DatabaseTrait};
use tdx_prover::repository::request_repository::{OnchainRequestRepositoryTrait, OnchainRequestId};
use hex::FromHex;
use anyhow::Result;

pub(crate) fn read_lines<P>(filename: P) -> Result<Vec<OnchainRequestId>>
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

pub(crate) async fn fetch_onchain_request_ids(status: Option<TdxQuoteStatus>, count: u64) -> Vec<OnchainRequestId> {
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
