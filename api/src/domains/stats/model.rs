use serde::Serialize;

#[derive(Debug, Clone, Copy, Default, Serialize, sqlx::FromRow)]
pub struct StatsCounts {
    pub latest_block: u64,
    pub total_blocks: u64,
    pub total_transactions: u64,
    pub total_logs: u64,
    pub total_addresses: u64,
    pub total_contracts: u64,
    pub total_tokens: u64,
    pub total_token_transfers: u64,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct StatsResponse {
    #[serde(flatten)]
    pub counts: StatsCounts,
    pub counts_updated_at_ms: i64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, sqlx::FromRow)]
pub struct PeriodCounts {
    pub total: u64,
    pub today: u64,
    pub yesterday: u64,
    pub this_week: u64,
    pub last_week: u64,
    pub this_month: u64,
    pub last_month: u64,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct NeoXStatistics {
    pub contracts: PeriodCounts,
    pub addresses: PeriodCounts,
    pub invocations: PeriodCounts,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct NeoXStatisticsResponse {
    pub updated_at_ms: i64,
    pub data: NeoXStatistics,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct NetworkStatistics {
    pub total_transactions: u64,
    pub total_addresses: u64,
    pub total_contracts: u64,
    pub current_week_transactions: u64,
    pub current_week_addresses: u64,
    pub current_week_contracts: u64,
}

#[derive(Debug, Clone, Copy, Default, sqlx::FromRow)]
pub struct RollingWeekCounts {
    pub transactions: u64,
    pub addresses: u64,
    pub contracts: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockWindow {
    pub start: u64,
    pub end: u64,
}

impl BlockWindow {
    pub const EMPTY: BlockWindow = BlockWindow { start: 1, end: 0 };
}
