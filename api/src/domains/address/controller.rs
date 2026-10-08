use actix_web::{get, web, HttpResponse};

use crate::domains::model::PageQuery;
use crate::errors::error::ApiError;
use crate::AppState;

use super::service;

#[get("/v1/neox/address")]
async fn list_addresses(
    pool: web::Data<AppState>,
    page: web::Query<PageQuery>,
) -> Result<HttpResponse, ApiError> {
    let addresses = service::list_addresses(&pool.connection, page.into_inner()).await?;

    Ok(HttpResponse::Ok().json(addresses))
}

#[get("/v1/neox/address/{address}")]
async fn get_address(
    pool: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let address = service::get_address(&pool.connection, path.into_inner()).await?;

    Ok(HttpResponse::Ok().json(address))
}

#[get("/v1/neox/address/{address}/transactions")]
async fn list_address_transactions(
    pool: web::Data<AppState>,
    path: web::Path<String>,
    page: web::Query<PageQuery>,
) -> Result<HttpResponse, ApiError> {
    let transactions =
        service::list_address_transactions(&pool.connection, path.into_inner(), page.into_inner())
            .await?;

    Ok(HttpResponse::Ok().json(transactions))
}

#[get("/v1/neox/address/{address}/transfers")]
async fn list_address_transfers(
    pool: web::Data<AppState>,
    path: web::Path<String>,
    page: web::Query<PageQuery>,
) -> Result<HttpResponse, ApiError> {
    let transfers =
        service::list_address_transfers(&pool.connection, path.into_inner(), page.into_inner())
            .await?;

    Ok(HttpResponse::Ok().json(transfers))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(list_addresses)
        .service(list_address_transactions)
        .service(list_address_transfers)
        .service(get_address);
}
