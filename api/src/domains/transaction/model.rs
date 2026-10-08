use serde::Serialize;

use crate::domains::model::Page;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct TransactionResponse {
    #[serde(skip)]
    pub page_cursor: u64,
    pub hash: String,
    pub block_number: u64,
    pub timestamp: u64,
    pub tx_index: u32,
    pub from_address: String,
    pub to_address: Option<String>,
    pub value: String,
    pub gas: u64,
    pub gas_price: Option<String>,
    pub max_fee_per_gas: Option<String>,
    pub max_priority_fee_per_gas: Option<String>,
    pub nonce: u64,
    pub tx_type: u16,
    pub input: String,
    pub status: Option<u8>,
    pub gas_used: Option<u64>,
    pub cumulative_gas_used: Option<u64>,
    pub effective_gas_price: Option<String>,
    pub contract_address: Option<String>,
}

pub type TransactionList = Page<TransactionResponse>;
