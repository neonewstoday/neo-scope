#[derive(Debug, Clone)]
pub struct Block {
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

#[derive(Debug, Clone)]
pub struct Transaction {
    pub hash: String,
    pub block_number: u64,
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

#[derive(Debug, Clone)]
pub struct Log {
    pub block_number: u64,
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

#[derive(Debug, Clone)]
pub struct Contract {
    pub address: String,
    pub creator_address: String,
    pub creation_tx_hash: String,
    pub creation_block: u64,
    pub creation_timestamp: u64,
    pub creation_status: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct Address {
    pub address: String,
    pub first_seen_block: u64,
    pub last_seen_block: u64,
    pub is_contract: bool,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub address: String,
    pub token_type: String,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub decimals: Option<u16>,
    pub first_seen_block: u64,
}

#[derive(Debug, Clone)]
pub struct TokenTransfer {
    pub block_number: u64,
    pub log_index: u32,
    pub sub_index: u32,
    pub tx_hash: String,
    pub tx_index: u32,
    pub token_address: String,
    pub token_type: String,
    pub operator: Option<String>,
    pub from_address: String,
    pub to_address: String,
    pub token_id: Option<String>,
    pub amount: String,
}

#[derive(Debug, Default)]
pub struct BlockBatch {
    pub blocks: Vec<Block>,
    pub transactions: Vec<Transaction>,
    pub logs: Vec<Log>,
    pub contracts: Vec<Contract>,
    pub addresses: Vec<Address>,
    pub tokens: Vec<Token>,
    pub token_transfers: Vec<TokenTransfer>,
}
