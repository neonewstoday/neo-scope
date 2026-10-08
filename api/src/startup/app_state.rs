use actix_web::web;
use anyhow::{Context, Result};
use sqlx::mysql::MySqlPoolOptions;

use crate::config::ApiConfig;
use crate::AppState;

pub async fn build_app_state(config: &ApiConfig) -> Result<web::Data<AppState>> {
    let pool = MySqlPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.database_url)
        .await
        .context("Failed to create MySQL pool")?;

    Ok(web::Data::new(AppState { connection: pool }))
}
