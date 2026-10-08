use anyhow::{bail, Result};
use log::{info, warn};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use crate::rpc::RpcClient;

pub async fn wait_for_node_ready(
    rpc: Arc<RpcClient>,
    expected_chain_id: Option<u64>,
) -> Result<()> {
    let max_retries = 12;
    let delay = Duration::from_secs(5);

    for attempt in 1..=max_retries {
        match rpc.client_version().await {
            Ok(version) => {
                let chain_id = rpc.chain_id().await?;

                if let Some(expected) = expected_chain_id {
                    if chain_id != expected {
                        bail!(
                            "Node chain id {} does not match configured CHAIN_ID {}. \
                             Check NODE_URL / CHAIN_ID.",
                            chain_id,
                            expected
                        );
                    }
                }

                info!("✅ NODE is ready: {} (chainId {})", version, chain_id);
                return Ok(());
            }
            Err(e) => {
                warn!(
                    "⏳ Attempt {}/{}: NODE not ready: {}. Retrying in {}s...",
                    attempt,
                    max_retries,
                    e,
                    delay.as_secs()
                );
            }
        }
        sleep(delay).await;
    }

    bail!(
        "❌ NODE did not become ready after {} attempts",
        max_retries
    )
}
