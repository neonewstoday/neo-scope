use actix_web::{get, web, HttpResponse};

use crate::domains::model::PageQuery;
use crate::errors::error::ApiError;
use crate::AppState;

use super::model::TokenListQuery;
use super::service;

#[get("/v1/neox/token")]
async fn list_tokens(
    pool: web::Data<AppState>,
    query: web::Query<TokenListQuery>,
) -> Result<HttpResponse, ApiError> {
    let tokens = service::list_tokens(&pool.connection, query.into_inner()).await?;

    Ok(HttpResponse::Ok().json(tokens))
}

#[get("/v1/neox/token/{address}")]
async fn get_token_by_address(
    pool: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let token = service::get_token(&pool.connection, path.into_inner()).await?;

    Ok(HttpResponse::Ok().json(token))
}

#[get("/v1/neox/token/{address}/transfers")]
async fn list_token_transfers(
    pool: web::Data<AppState>,
    path: web::Path<String>,
    page: web::Query<PageQuery>,
) -> Result<HttpResponse, ApiError> {
    let transfers =
        service::list_token_transfers(&pool.connection, path.into_inner(), page.into_inner())
            .await?;

    Ok(HttpResponse::Ok().json(transfers))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(list_tokens)
        .service(list_token_transfers)
        .service(get_token_by_address);
}
