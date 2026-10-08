#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub node_url: String,
    pub chain_id: Option<u64>,
    pub batch_size: u64,
    pub keep_alive_interval: u64,
    pub start_block: u64,
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set in .env or environment"),
            node_url: std::env::var("NODE_URL")
                .expect("NODE_URL must be set in .env or environment"),
            chain_id: std::env::var("CHAIN_ID")
                .ok()
                .and_then(|v| v.trim().parse::<u64>().ok()),
            batch_size: parse_env_u64("BATCH_SIZE", 25),
            keep_alive_interval: parse_env_u64("KEEP_ALIVE_INTERVAL", 5),
            start_block: parse_env_u64("START_BLOCK", 0),
        }
    }
}

fn parse_env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .unwrap_or(default)
}
