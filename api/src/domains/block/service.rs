use sqlx::{MySql, Pool};

use crate::domains::model::PageQuery;
use crate::domains::transaction::{self, model::TransactionList};
use crate::errors::error::ApiError;
use crate::utils::validate::normalize_hash;

use super::model::{BlockList, BlockResponse};
use super::query;

pub async fn list_blocks(conn: &Pool<MySql>, page: PageQuery) -> Result<BlockList, ApiError> {
    query::list_blocks(conn, page).await
}

pub async fn get_block(conn: &Pool<MySql>, id: String) -> Result<BlockResponse, ApiError> {
    let block = match id.trim().parse::<u64>() {
        Ok(number) => query::find_block_by_number(conn, number).await?,
        Err(_) => query::find_block_by_hash(conn, &parse_block_hash(&id)?).await?,
    };

    block.ok_or_else(|| ApiError::not_found("Block does not exist."))
}

pub async fn list_block_transactions(
    conn: &Pool<MySql>,
    id: String,
    page: PageQuery,
) -> Result<TransactionList, ApiError> {
    let block_number = resolve_block_number(conn, &id).await?;

    transaction::query::list_transactions_by_block(conn, block_number, page).await
}

async fn resolve_block_number(conn: &Pool<MySql>, id: &str) -> Result<u64, ApiError> {
    let number = match id.trim().parse::<u64>() {
        Ok(number) => query::block_exists(conn, number).await?.then_some(number),
        Err(_) => query::find_block_number_by_hash(conn, &parse_block_hash(id)?).await?,
    };

    number.ok_or_else(|| ApiError::not_found("Block does not exist."))
}

fn parse_block_hash(id: &str) -> Result<String, ApiError> {
    normalize_hash(id).ok_or_else(|| ApiError::bad_request("Invalid block number or hash."))
}
