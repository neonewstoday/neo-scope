use chrono::Utc;
use log::{error, info};
use sqlx::{MySql, Pool};

use crate::domains::block;
use crate::errors::error::ApiError;
use crate::utils::datetime::now_timestamp_ms;

use super::model::{NeoXStatistics, NeoXStatisticsResponse, NetworkStatistics, StatsResponse};
use super::period::Periods;
use super::query;
use super::state::{CURRENT_NEOX_STATISTICS, CURRENT_NETWORK_STATISTICS, CURRENT_STATS};

pub async fn get_stats(conn: &Pool<MySql>) -> Result<StatsResponse, ApiError> {
    let mut stats = cached_stats();
    stats.counts.latest_block = query::get_latest_block(conn).await?;

    Ok(stats)
}

pub fn get_neox_statistics() -> NeoXStatisticsResponse {
    *CURRENT_NEOX_STATISTICS.read().unwrap()
}

pub fn get_network_statistics() -> NetworkStatistics {
    *CURRENT_NETWORK_STATISTICS.read().unwrap()
}

pub async fn refresh_cached_stats(conn: &Pool<MySql>) -> bool {
    let latest_block = match query::get_latest_block(conn).await {
        Ok(latest_block) => latest_block,
        Err(e) => {
            error!("Failed to read the current height: {}", e);
            return false;
        }
    };

    let cached = cached_stats();
    if cached.counts_updated_at_ms != 0 && cached.counts.latest_block == latest_block {
        return true;
    }

    match query::get_counts(conn).await {
        Ok(counts) => {
            let mut writer = CURRENT_STATS.write().unwrap();
            writer.counts = counts;
            writer.counts_updated_at_ms = now_timestamp_ms();
            info!("Stats refreshed. Current height is {}.", latest_block);
            true
        }
        Err(e) => {
            error!("Failed to refresh the stats cache: {}", e);
            false
        }
    }
}

pub async fn refresh_cached_statistics(conn: &Pool<MySql>) -> bool {
    match compute_statistics(conn).await {
        Ok((statistics, network)) => {
            {
                let mut writer = CURRENT_NEOX_STATISTICS.write().unwrap();
                writer.updated_at_ms = now_timestamp_ms();
                writer.data = statistics;
            }
            {
                let mut writer = CURRENT_NETWORK_STATISTICS.write().unwrap();
                *writer = network;
            }
            info!("Statistics refreshed.");
            true
        }
        Err(e) => {
            error!("Failed to refresh the statistics cache: {}", e);
            false
        }
    }
}

async fn compute_statistics(
    conn: &Pool<MySql>,
) -> Result<(NeoXStatistics, NetworkStatistics), ApiError> {
    let periods = Periods::at(Utc::now());
    let windows = query::resolve_windows(conn, &periods).await?;
    let rolling_week_start =
        block::query::first_block_at_or_after(conn, periods.rolling_week_start).await?;

    let (contracts, addresses, invocations, total_transactions, week) = tokio::try_join!(
        query::get_contract_period_counts(conn, &windows),
        query::get_address_period_counts(conn, &windows),
        query::get_invocation_period_counts(conn, &windows),
        query::count_transactions(conn),
        query::get_rolling_week_counts(conn, rolling_week_start),
    )?;

    let statistics = NeoXStatistics {
        contracts,
        addresses,
        invocations,
    };

    let network = NetworkStatistics {
        total_transactions,
        total_addresses: addresses.total,
        total_contracts: contracts.total,
        current_week_transactions: week.transactions,
        current_week_addresses: week.addresses,
        current_week_contracts: week.contracts,
    };

    Ok((statistics, network))
}

fn cached_stats() -> StatsResponse {
    *CURRENT_STATS.read().unwrap()
}
