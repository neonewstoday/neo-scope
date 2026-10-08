mod config;
mod engine;
mod rpc;
mod startup;
mod util;

use anyhow::Result;
use log::info;
use std::sync::Arc;

use config::AppConfig;
use engine::Indexer;
use neo_scope_db::db::Database;
use rpc::RpcClient;
use startup::{init_logger, wait_for_node_ready};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    init_logger();

    let config = AppConfig::new();

    if let Err(e) = run(config).await {
        eprintln!("Application error: {:?}", e);
        std::process::exit(1);
    }
}

async fn run(config: AppConfig) -> Result<()> {
    info!(
        "Initializing RPC client (Neo X node: {})...",
        config.node_url
    );
    let rpc = Arc::new(RpcClient::new(&config.node_url));

    wait_for_node_ready(rpc.clone(), config.chain_id).await?;

    info!("Initializing database...");
    let db = Database::new(&config.database_url).await?;

    let indexer = Indexer::new(rpc, db, config);
    indexer.run().await
}
