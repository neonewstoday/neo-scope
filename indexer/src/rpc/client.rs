use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use reqwest::Client as ReqwestClient;
use serde::de::DeserializeOwned;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

use super::model::{EvmBlock, EvmReceipt, RpcResponse};
use crate::util::hex_to_u64;

static SEMAPHORE: Lazy<Arc<Semaphore>> = Lazy::new(|| Arc::new(Semaphore::new(10)));

static HTTP_CLIENT: Lazy<ReqwestClient> = Lazy::new(|| {
    ReqwestClient::builder()
        .pool_idle_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
});

pub struct RpcClient {
    base_url: String,
}

impl RpcClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
        }
    }

    async fn call<R: DeserializeOwned>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<R> {
        let _permit = SEMAPHORE.acquire().await.expect("Semaphore closed");

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });

        let response: RpcResponse<R> = HTTP_CLIENT
            .post(&self.base_url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        if let Some(err) = response.error {
            return Err(anyhow!(
                "RPC error on {} ({}): {}",
                method,
                err.code,
                err.message
            ));
        }
        response
            .result
            .ok_or_else(|| anyhow!("RPC {} returned no result", method))
    }

    pub async fn block_number(&self) -> Result<u64> {
        let hex: String = self.call("eth_blockNumber", json!([])).await?;
        hex_to_u64(&hex)
    }

    pub async fn chain_id(&self) -> Result<u64> {
        let hex: String = self.call("eth_chainId", json!([])).await?;
        hex_to_u64(&hex)
    }

    pub async fn client_version(&self) -> Result<String> {
        self.call("web3_clientVersion", json!([])).await
    }

    pub async fn get_block_by_number(&self, number: u64) -> Result<EvmBlock> {
        let params = json!([format!("0x{:x}", number), true]);
        self.call("eth_getBlockByNumber", params).await
    }

    pub async fn get_block_receipts(&self, number: u64) -> Result<Vec<EvmReceipt>> {
        let params = json!([format!("0x{:x}", number)]);
        self.call("eth_getBlockReceipts", params).await
    }

    pub async fn eth_call(&self, to: &str, data: &str) -> Result<String> {
        let params = json!([{ "to": to, "data": data }, "latest"]);
        self.call("eth_call", params).await
    }
}
