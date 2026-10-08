use actix_web::web;
use log::info;
use std::time::Duration;
use tokio::{
    task,
    time::{self, MissedTickBehavior},
};

use crate::config::ApiConfig;
use crate::domains::stats;
use crate::AppState;

const RETRY_AFTER_FAILURE: Duration = Duration::from_secs(10);

pub fn start_periodic_tasks(app_state: web::Data<AppState>, config: &ApiConfig) {
    info!(
        "Stats cache refresh: first run immediately, then every {}s (retry after 10s on failure).",
        config.stats_refresh_interval
    );
    info!(
        "Statistics cache refresh: first run immediately, then every {}s (retry after 10s on failure).",
        config.statistics_refresh_interval
    );

    let counts_state = app_state.clone();
    let counts_interval = config.stats_refresh_interval;
    task::spawn(async move {
        let mut interval = ticker(counts_interval);
        loop {
            interval.tick().await;
            while !stats::service::refresh_cached_stats(&counts_state.connection).await {
                time::sleep(RETRY_AFTER_FAILURE).await;
            }
        }
    });

    let statistics_interval = config.statistics_refresh_interval;
    task::spawn(async move {
        let mut interval = ticker(statistics_interval);
        loop {
            interval.tick().await;
            while !stats::service::refresh_cached_statistics(&app_state.connection).await {
                time::sleep(RETRY_AFTER_FAILURE).await;
            }
        }
    });
}

fn ticker(interval_secs: u64) -> time::Interval {
    let mut interval = time::interval(Duration::from_secs(interval_secs));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    interval
}
