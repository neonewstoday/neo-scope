use actix_web::{get, web, HttpResponse};

use crate::errors::error::ApiError;
use crate::AppState;

use super::service;

#[get("/v1/neox/stats")]
async fn get_stats(pool: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    let stats = service::get_stats(&pool.connection).await?;

    Ok(HttpResponse::Ok().json(stats))
}

#[get("/v1/neox/stat/neox-statistics")]
async fn get_neox_statistics() -> HttpResponse {
    HttpResponse::Ok().json(service::get_neox_statistics())
}

#[get("/v1/neox/stat/network-statistics")]
async fn get_network_statistics() -> HttpResponse {
    HttpResponse::Ok().json(service::get_network_statistics())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_stats)
        .service(get_neox_statistics)
        .service(get_network_statistics);
}
