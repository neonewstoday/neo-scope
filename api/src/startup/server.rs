use actix_cors::Cors;
use actix_web::{http::header, web, App, HttpResponse, HttpServer};
use log::info;

use crate::config::ApiConfig;
use crate::domains::log::controller as log_controller;
use crate::domains::{address, block, contract, stats, token, transaction};
use crate::errors::error::Error;
use crate::AppState;

pub async fn start_http_server(
    app_state: web::Data<AppState>,
    config: &ApiConfig,
) -> std::io::Result<()> {
    info!(
        "Opening to requests on http://{}:{}/v1/neox.",
        config.host, config.port
    );

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET"])
            .allowed_header(header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .app_data(query_config())
            .app_data(path_config())
            .configure(block::controller::config)
            .configure(transaction::controller::config)
            .configure(address::controller::config)
            .configure(contract::controller::config)
            .configure(token::controller::config)
            .configure(log_controller::config)
            .configure(stats::controller::config)
            .default_service(web::to(not_found))
    })
    .bind((config.host.as_str(), config.port))?
    .run()
    .await
}

fn query_config() -> web::QueryConfig {
    web::QueryConfig::default().error_handler(|err, _| {
        actix_web::error::InternalError::from_response(
            "",
            HttpResponse::BadRequest().json(Error {
                error: format!("Invalid query parameters: {err}"),
            }),
        )
        .into()
    })
}

fn path_config() -> web::PathConfig {
    web::PathConfig::default().error_handler(|err, _| {
        actix_web::error::InternalError::from_response(
            "",
            HttpResponse::BadRequest().json(Error {
                error: format!("Invalid path parameters: {err}"),
            }),
        )
        .into()
    })
}

async fn not_found() -> HttpResponse {
    HttpResponse::NotFound().json(Error {
        error: "Endpoint does not exist.".to_string(),
    })
}
