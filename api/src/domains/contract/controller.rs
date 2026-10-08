use actix_web::{get, web, HttpResponse};

use crate::domains::model::PageQuery;
use crate::errors::error::ApiError;
use crate::AppState;

use super::model::InvocationCountQuery;
use super::service;

#[get("/v1/neox/contract")]
async fn list_contracts(
    pool: web::Data<AppState>,
    page: web::Query<PageQuery>,
) -> Result<HttpResponse, ApiError> {
    let contracts = service::list_contracts(&pool.connection, page.into_inner()).await?;

    Ok(HttpResponse::Ok().json(contracts))
}

#[get("/v1/neox/contract/{address}/invocations")]
async fn count_contract_invocations(
    pool: web::Data<AppState>,
    path: web::Path<String>,
    query: web::Query<InvocationCountQuery>,
) -> Result<HttpResponse, ApiError> {
    let invocations = service::count_contract_invocations(
        &pool.connection,
        path.into_inner(),
        query.into_inner(),
    )
    .await?;

    Ok(HttpResponse::Ok().json(invocations))
}

#[get("/v1/neox/contract/{address}")]
async fn get_contract_by_address(
    pool: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let contract = service::get_contract(&pool.connection, path.into_inner()).await?;

    Ok(HttpResponse::Ok().json(contract))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(list_contracts)
        .service(count_contract_invocations)
        .service(get_contract_by_address);
}
