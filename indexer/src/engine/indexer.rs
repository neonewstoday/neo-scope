use anyhow::Result;
use log::{info, warn};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use crate::config::AppConfig;
use crate::engine::sync::sync_between;
use crate::engine::tokens::TokenCache;
use crate::rpc::RpcClient;
use neo_scope_db::db::Database;

pub struct Indexer {
    rpc: Arc<RpcClient>,
    db: Database,
    config: AppConfig,
}

impl Indexer {
    pub fn new(rpc: Arc<RpcClient>, db: Database, config: AppConfig) -> Self {
        Self { rpc, db, config }
    }

    pub async fn run(&self) -> Result<()> {
        let mut next = match self.db.max_block_number().await? {
            Some(n) => n + 1,
            None => self.config.start_block,
        };

        let known_tokens = self.db.known_token_addresses().await?;
        info!("Loaded {} known token contract(s)", known_tokens.len());
        let tokens = TokenCache::new(known_tokens);

        info!("🚀 Starting indexer at block {}", next);

        let mut announced_caught_up = false;

        loop {
            let tip = self.rpc.block_number().await?;

            if next > tip {
                if !announced_caught_up {
                    info!(
                        "Caught up at block {}. Following chain every {}s...",
                        tip, self.config.keep_alive_interval
                    );
                    announced_caught_up = true;
                }
                sleep(Duration::from_secs(self.config.keep_alive_interval)).await;
                continue;
            }
            announced_caught_up = false;

            let end = std::cmp::min(next + self.config.batch_size, tip + 1);
            let stats = sync_between(&self.rpc, &self.db, &tokens, next, end).await?;

            if stats.next_block == next {
                warn!(
                    "No blocks indexed from {}..{}; retrying in {}s",
                    next,
                    end - 1,
                    self.config.keep_alive_interval
                );
                sleep(Duration::from_secs(self.config.keep_alive_interval)).await;
                continue;
            }

            info!(
                "[blocks {}..{}] indexed {} block(s), {} tx, {} log(s), {} contract(s), \
                 {} transfer(s), {} new token(s)",
                next,
                stats.next_block - 1,
                stats.blocks,
                stats.transactions,
                stats.logs,
                stats.contracts,
                stats.token_transfers,
                stats.new_tokens
            );
            next = stats.next_block;
        }
    }
}
