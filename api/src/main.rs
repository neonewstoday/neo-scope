mod config;
mod domains;
mod errors;
mod startup;
mod utils;

use anyhow::Result;
use log::info;

use config::ApiConfig;
use startup::{build_app_state, init_logger, start_http_server, start_periodic_tasks};

pub struct AppState {
    pub connection: sqlx::Pool<sqlx::MySql>,
}

#[actix_web::main]
async fn main() {
    dotenvy::dotenv().ok();
    init_logger();

    if let Err(e) = run(ApiConfig::new()).await {
        eprintln!("Application error: {:?}", e);
        std::process::exit(1);
    }
}

async fn run(config: ApiConfig) -> Result<()> {
    info!("Building app state...");
    let app_state = build_app_state(&config).await?;

    info!("Starting periodic tasks...");
    start_periodic_tasks(app_state.clone(), &config);

    info!("Starting HTTP server...");
    start_http_server(app_state, &config).await?;

    Ok(())
}
