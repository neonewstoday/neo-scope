use actix_web::{get, web, HttpResponse};

use crate::domains::model::PageQuery;
use crate::errors::error::ApiError;
use crate::AppState;

use super::service;

#[get("/v1/neox/block")]
async fn list_blocks(
    pool: web::Data<AppState>,
    page: web::Query<PageQuery>,
) -> Result<HttpResponse, ApiError> {
    let blocks = service::list_blocks(&pool.connection, page.into_inner()).await?;

    Ok(HttpResponse::Ok().json(blocks))
}

#[get("/v1/neox/block/{id}")]
async fn get_block_by_id(
    pool: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let block = service::get_block(&pool.connection, path.into_inner()).await?;

    Ok(HttpResponse::Ok().json(block))
}

#[get("/v1/neox/block/{id}/transactions")]
async fn list_block_transactions(
    pool: web::Data<AppState>,
    path: web::Path<String>,
    page: web::Query<PageQuery>,
) -> Result<HttpResponse, ApiError> {
    let transactions =
        service::list_block_transactions(&pool.connection, path.into_inner(), page.into_inner())
            .await?;

    Ok(HttpResponse::Ok().json(transactions))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(list_blocks)
        .service(list_block_transactions)
        .service(get_block_by_id);
}
