use actix_web::{get, web, HttpResponse};

use crate::errors::error::ApiError;
use crate::AppState;

use super::model::LogListQuery;
use super::service;

#[get("/v1/neox/log")]
async fn list_logs(
    pool: web::Data<AppState>,
    query: web::Query<LogListQuery>,
) -> Result<HttpResponse, ApiError> {
    let logs = service::list_logs(&pool.connection, query.into_inner()).await?;

    Ok(HttpResponse::Ok().json(logs))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(list_logs);
}
