use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::domains::model::Page;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ContractResponse {
    #[serde(skip)]
    pub page_cursor: u64,
    pub address: String,
    pub creator_address: String,
    pub creation_tx_hash: String,
    pub creation_block: u64,
    pub creation_timestamp: u64,
    pub creation_status: Option<u8>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct InvocationCountQuery {
    #[serde(alias = "start")]
    pub started_at: Option<String>,
    #[serde(alias = "end")]
    pub ended_at: Option<String>,
}

pub type ContractList = Page<ContractResponse>;
pub type InvocationCounts = BTreeMap<String, u64>;
