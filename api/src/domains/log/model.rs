use serde::{Deserialize, Serialize};

use crate::domains::model::{BlockRange, Page, PageQuery};

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct LogResponse {
    #[serde(skip)]
    pub page_cursor: u64,
    pub block_number: u64,
    pub timestamp: u64,
    pub log_index: u32,
    pub tx_hash: String,
    pub tx_index: u32,
    pub address: String,
    pub topic0: Option<String>,
    pub topic1: Option<String>,
    pub topic2: Option<String>,
    pub topic3: Option<String>,
    pub data: String,
    pub removed: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct LogListQuery {
    pub id: Option<u64>,
    pub limit: Option<usize>,
    pub address: Option<String>,
    pub topic0: Option<String>,
    pub from_block: Option<u64>,
    pub to_block: Option<u64>,
    #[serde(alias = "start")]
    pub started_at: Option<String>,
    #[serde(alias = "end")]
    pub ended_at: Option<String>,
}

impl LogListQuery {
    pub fn page(&self) -> PageQuery {
        PageQuery {
            id: self.id,
            limit: self.limit,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LogFilter {
    pub address: Option<String>,
    pub topic0: Option<String>,
    pub blocks: BlockRange,
}

pub type LogList = Page<LogResponse>;
