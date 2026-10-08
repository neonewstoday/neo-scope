use actix_web::{get, web, HttpResponse};

use crate::domains::model::PageQuery;
use crate::errors::error::ApiError;
use crate::AppState;

use super::service;

#[get("/v1/neox/transaction")]
async fn list_transactions(
    pool: web::Data<AppState>,
    page: web::Query<PageQuery>,
) -> Result<HttpResponse, ApiError> {
    let transactions = service::list_transactions(&pool.connection, page.into_inner()).await?;

    Ok(HttpResponse::Ok().json(transactions))
}

#[get("/v1/neox/transaction/{hash}")]
async fn get_transaction_by_hash(
    pool: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let transaction = service::get_transaction(&pool.connection, path.into_inner()).await?;

    Ok(HttpResponse::Ok().json(transaction))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(list_transactions)
        .service(get_transaction_by_hash);
}
