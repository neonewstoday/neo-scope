use anyhow::Result;
use futures::future::join_all;
use log::warn;
use std::collections::HashMap;
use std::sync::Arc;

use crate::engine::converters::convert_batch;
use crate::engine::tokens::{resolve_new_tokens, TokenCache};
use crate::rpc::model::{EvmBlock, EvmReceipt};
use crate::rpc::RpcClient;
use crate::util::hex_to_u64;
use neo_scope_db::db::Database;

#[derive(Debug, Default, Clone, Copy)]
pub struct SyncStats {
    pub next_block: u64,
    pub blocks: usize,
    pub transactions: usize,
    pub logs: usize,
    pub contracts: usize,
    pub token_transfers: usize,
    pub new_tokens: usize,
}

pub async fn sync_between(
    rpc: &Arc<RpcClient>,
    db: &Database,
    tokens: &TokenCache,
    start: u64,
    end: u64,
) -> Result<SyncStats> {
    let block_futures = (start..end).map(|n| rpc.get_block_by_number(n));
    let block_results = join_all(block_futures).await;

    let mut blocks_ok: Vec<(u64, EvmBlock)> = Vec::new();
    for (offset, res) in block_results.into_iter().enumerate() {
        match res {
            Ok(b) => {
                let number = hex_to_u64(&b.number)?;
                blocks_ok.push((number, b));
            }
            Err(e) => {
                warn!(
                    "⚠️  Block {} failed: {:?}. Stopping this batch here",
                    start + offset as u64,
                    e
                );
                break;
            }
        }
    }

    let receipt_futures = blocks_ok
        .iter()
        .filter(|(_, b)| !b.transactions.is_empty())
        .map(|(n, _)| {
            let n = *n;
            async move { (n, rpc.get_block_receipts(n).await) }
        });
    let receipt_results = join_all(receipt_futures).await;

    let mut receipts_by_block: HashMap<u64, Vec<EvmReceipt>> = HashMap::new();
    let mut first_failed_receipts: Option<u64> = None;
    for (n, res) in receipt_results {
        match res {
            Ok(receipts) => {
                receipts_by_block.insert(n, receipts);
            }
            Err(e) => {
                warn!("⚠️  Receipts for block {} failed: {:?}", n, e);
                first_failed_receipts = Some(first_failed_receipts.map_or(n, |m: u64| m.min(n)));
            }
        }
    }

    if let Some(n) = first_failed_receipts {
        blocks_ok.retain(|(number, _)| *number < n);
    }

    if blocks_ok.is_empty() {
        return Ok(SyncStats {
            next_block: start,
            ..SyncStats::default()
        });
    }

    let mut batch = convert_batch(&blocks_ok, &receipts_by_block)?;

    batch.tokens = resolve_new_tokens(rpc, tokens, &batch.token_transfers).await;

    db.insert_batch(&batch).await?;

    Ok(SyncStats {
        next_block: start + batch.blocks.len() as u64,
        blocks: batch.blocks.len(),
        transactions: batch.transactions.len(),
        logs: batch.logs.len(),
        contracts: batch.contracts.len(),
        token_transfers: batch.token_transfers.len(),
        new_tokens: batch.tokens.len(),
    })
}
