#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
    pub stats_refresh_interval: u64,
    pub statistics_refresh_interval: u64,
}

impl ApiConfig {
    pub fn new() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set in .env or environment"),
            host: std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: parse_env("API_PORT", 8083),
            max_connections: parse_env("API_MAX_CONNECTIONS", 10),
            stats_refresh_interval: parse_env("STATS_REFRESH_INTERVAL", 300),
            statistics_refresh_interval: parse_env("STATISTICS_REFRESH_INTERVAL", 900),
        }
    }
}

fn parse_env<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse::<T>().ok())
        .unwrap_or(default)
}
