use once_cell::sync::Lazy;
use std::sync::RwLock;

use super::model::{NeoXStatisticsResponse, NetworkStatistics, StatsResponse};

pub static CURRENT_STATS: Lazy<RwLock<StatsResponse>> =
    Lazy::new(|| RwLock::new(StatsResponse::default()));

pub static CURRENT_NEOX_STATISTICS: Lazy<RwLock<NeoXStatisticsResponse>> =
    Lazy::new(|| RwLock::new(NeoXStatisticsResponse::default()));

pub static CURRENT_NETWORK_STATISTICS: Lazy<RwLock<NetworkStatistics>> =
    Lazy::new(|| RwLock::new(NetworkStatistics::default()));
