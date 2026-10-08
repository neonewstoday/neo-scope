use sqlx::{MySql, Pool};

use crate::domains::model::PageQuery;
use crate::errors::error::ApiError;
use crate::utils::validate::normalize_hash;

use super::model::{TransactionList, TransactionResponse};
use super::query;

pub async fn list_transactions(
    conn: &Pool<MySql>,
    page: PageQuery,
) -> Result<TransactionList, ApiError> {
    query::list_transactions(conn, page).await
}

pub async fn get_transaction(
    conn: &Pool<MySql>,
    hash: String,
) -> Result<TransactionResponse, ApiError> {
    let hash =
        normalize_hash(&hash).ok_or_else(|| ApiError::bad_request("Invalid transaction hash."))?;

    query::find_transaction_by_hash(conn, &hash)
        .await?
        .ok_or_else(|| ApiError::not_found("Transaction does not exist."))
}
