use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RpcResponse<T> {
    pub result: Option<T>,
    pub error: Option<RpcError>,
}

#[derive(Deserialize, Debug)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EvmBlock {
    pub number: String,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: String,
    pub miner: String,
    pub gas_used: String,
    pub gas_limit: String,
    #[serde(default)]
    pub base_fee_per_gas: Option<String>,
    pub size: String,
    #[serde(default)]
    pub extra_data: Option<String>,
    #[serde(default)]
    pub transactions: Vec<EvmTransaction>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EvmTransaction {
    pub hash: String,
    pub from: String,
    #[serde(default)]
    pub to: Option<String>,
    pub value: String,
    pub gas: String,
    #[serde(default)]
    pub gas_price: Option<String>,
    #[serde(default)]
    pub max_fee_per_gas: Option<String>,
    #[serde(default)]
    pub max_priority_fee_per_gas: Option<String>,
    pub nonce: String,
    pub transaction_index: String,
    #[serde(rename = "type", default)]
    pub tx_type: Option<String>,
    pub input: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EvmReceipt {
    pub transaction_hash: String,
    #[serde(default)]
    pub transaction_index: Option<String>,
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    pub gas_used: String,
    pub cumulative_gas_used: String,
    #[serde(default)]
    pub effective_gas_price: Option<String>,
    #[serde(default)]
    pub contract_address: Option<String>,
    #[serde(default)]
    pub logs: Vec<EvmLog>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EvmLog {
    pub address: String,
    #[serde(default)]
    pub topics: Vec<String>,
    pub data: String,
    #[serde(default)]
    pub transaction_hash: Option<String>,
    #[serde(default)]
    pub transaction_index: Option<String>,
    pub log_index: String,
    #[serde(default)]
    pub removed: bool,
}
