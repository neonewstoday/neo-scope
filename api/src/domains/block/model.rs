use serde::Serialize;

use crate::domains::model::Page;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct BlockResponse {
    pub number: u64,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: u64,
    pub miner: String,
    pub gas_used: u64,
    pub gas_limit: u64,
    pub base_fee_per_gas: Option<String>,
    pub size: u64,
    pub tx_count: u32,
    pub extra_data: Option<String>,
}

pub type BlockList = Page<BlockResponse>;
