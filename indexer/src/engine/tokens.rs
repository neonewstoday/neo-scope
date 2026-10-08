use futures::future::join_all;
use log::debug;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::abi::{decode_small_uint, decode_string};
use super::token_transfers::TOKEN_TYPE_ERC20;
use crate::rpc::RpcClient;
use neo_scope_db::db::model::{Token, TokenTransfer};

const SELECTOR_NAME: &str = "0x06fdde03";
const SELECTOR_SYMBOL: &str = "0x95d89b41";
const SELECTOR_DECIMALS: &str = "0x313ce567";

pub struct TokenCache {
    described: Mutex<HashSet<String>>,
}

impl TokenCache {
    pub fn new(known: impl IntoIterator<Item = String>) -> Self {
        Self {
            described: Mutex::new(known.into_iter().collect()),
        }
    }
}

pub async fn resolve_new_tokens(
    rpc: &Arc<RpcClient>,
    cache: &TokenCache,
    transfers: &[TokenTransfer],
) -> Vec<Token> {
    let mut candidates: HashMap<&str, (&str, u64)> = HashMap::new();
    for t in transfers {
        candidates
            .entry(&t.token_address)
            .and_modify(|(_, block)| *block = (*block).min(t.block_number))
            .or_insert((&t.token_type, t.block_number));
    }

    let pending: Vec<(String, String, u64)> = {
        let mut described = cache.described.lock().await;
        candidates
            .into_iter()
            .filter_map(|(address, (token_type, block))| {
                if !described.insert(address.to_string()) {
                    return None;
                }
                Some((address.to_string(), token_type.to_string(), block))
            })
            .collect()
    };

    if pending.is_empty() {
        return Vec::new();
    }

    let futures = pending
        .into_iter()
        .map(|(address, token_type, block)| fetch_metadata(rpc, address, token_type, block));
    join_all(futures).await
}

async fn fetch_metadata(
    rpc: &Arc<RpcClient>,
    address: String,
    token_type: String,
    first_seen_block: u64,
) -> Token {
    let name = call_string(rpc, &address, SELECTOR_NAME).await;
    let symbol = call_string(rpc, &address, SELECTOR_SYMBOL).await;
    let decimals = if token_type == TOKEN_TYPE_ERC20 {
        call_uint(rpc, &address, SELECTOR_DECIMALS).await
    } else {
        None
    };

    debug!(
        "Token {} ({}) name={:?} symbol={:?} decimals={:?}",
        address, token_type, name, symbol, decimals
    );

    Token {
        address,
        token_type,
        name,
        symbol,
        decimals,
        first_seen_block,
    }
}

async fn call_string(rpc: &Arc<RpcClient>, address: &str, selector: &str) -> Option<String> {
    match rpc.eth_call(address, selector).await {
        Ok(ret) => decode_string(&ret).map(truncate),
        Err(_) => None,
    }
}

async fn call_uint(rpc: &Arc<RpcClient>, address: &str, selector: &str) -> Option<u16> {
    match rpc.eth_call(address, selector).await {
        Ok(ret) => decode_small_uint(&ret),
        Err(_) => None,
    }
}

fn truncate(s: String) -> String {
    if s.chars().count() <= 255 {
        return s;
    }
    s.chars().take(255).collect()
}
