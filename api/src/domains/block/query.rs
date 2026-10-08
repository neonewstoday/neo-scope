use sqlx::{AssertSqlSafe, MySql, Pool};

use crate::domains::model::{BlockRange, Page, PageQuery, TimeRange};
use crate::errors::error::ApiError;

use super::model::{BlockList, BlockResponse};

const SELECT_BLOCK: &str = "SELECT `number`, `hash`, `parentHash` AS parent_hash, `timestamp`, \
    `miner`, `gasUsed` AS gas_used, `gasLimit` AS gas_limit, \
    `baseFeePerGas` AS base_fee_per_gas, `size`, `txCount` AS tx_count, \
    `extraData` AS extra_data FROM `block`";

pub async fn find_block_by_number(
    conn: &Pool<MySql>,
    number: u64,
) -> Result<Option<BlockResponse>, ApiError> {
    let sql = format!("{SELECT_BLOCK} WHERE `number` = ?");

    let block = sqlx::query_as::<_, BlockResponse>(AssertSqlSafe(sql))
        .bind(number)
        .fetch_optional(conn)
        .await?;

    Ok(block)
}

pub async fn find_block_by_hash(
    conn: &Pool<MySql>,
    hash: &str,
) -> Result<Option<BlockResponse>, ApiError> {
    let sql = format!("{SELECT_BLOCK} WHERE `hash` = ?");

    let block = sqlx::query_as::<_, BlockResponse>(AssertSqlSafe(sql))
        .bind(hash)
        .fetch_optional(conn)
        .await?;

    Ok(block)
}

pub async fn find_block_number_by_hash(
    conn: &Pool<MySql>,
    hash: &str,
) -> Result<Option<u64>, ApiError> {
    let number = sqlx::query_scalar::<_, u64>("SELECT `number` FROM `block` WHERE `hash` = ?")
        .bind(hash)
        .fetch_optional(conn)
        .await?;

    Ok(number)
}

pub async fn block_exists(conn: &Pool<MySql>, number: u64) -> Result<bool, ApiError> {
    let found = sqlx::query_scalar::<_, u64>("SELECT `number` FROM `block` WHERE `number` = ?")
        .bind(number)
        .fetch_optional(conn)
        .await?;

    Ok(found.is_some())
}

pub async fn first_block_at_or_after(
    conn: &Pool<MySql>,
    timestamp: i64,
) -> Result<Option<u64>, ApiError> {
    let timestamp = u64::try_from(timestamp).unwrap_or(0);

    let number = sqlx::query_scalar::<_, u64>(
        "SELECT `number` FROM `block` WHERE `timestamp` >= ? ORDER BY `timestamp` ASC LIMIT 1",
    )
    .bind(timestamp)
    .fetch_optional(conn)
    .await?;

    Ok(number)
}

pub async fn last_block_at_or_before(
    conn: &Pool<MySql>,
    timestamp: i64,
) -> Result<Option<u64>, ApiError> {
    let Ok(timestamp) = u64::try_from(timestamp) else {
        return Ok(None);
    };

    let number = sqlx::query_scalar::<_, u64>(
        "SELECT `number` FROM `block` WHERE `timestamp` <= ? ORDER BY `timestamp` DESC LIMIT 1",
    )
    .bind(timestamp)
    .fetch_optional(conn)
    .await?;

    Ok(number)
}

pub async fn resolve_block_range(
    conn: &Pool<MySql>,
    range: TimeRange,
) -> Result<BlockRange, ApiError> {
    let from = match range.started_at {
        Some(at) => match first_block_at_or_after(conn, at).await? {
            Some(number) => Some(number),
            None => return Ok(BlockRange::EMPTY),
        },
        None => None,
    };

    let to = match range.ended_at {
        Some(at) => match last_block_at_or_before(conn, at).await? {
            Some(number) => Some(number),
            None => return Ok(BlockRange::EMPTY),
        },
        None => None,
    };

    Ok(BlockRange { from, to })
}

pub async fn list_blocks(conn: &Pool<MySql>, page: PageQuery) -> Result<BlockList, ApiError> {
    let mut sql = String::from(SELECT_BLOCK);
    if page.cursor().is_some() {
        sql.push_str(" WHERE `number` < ?");
    }
    sql.push_str(" ORDER BY `number` DESC LIMIT ?");

    let mut query = sqlx::query_as::<_, BlockResponse>(AssertSqlSafe(sql));
    if let Some(cursor) = page.cursor() {
        query = query.bind(cursor);
    }

    let rows = query.bind(page.fetch_size()).fetch_all(conn).await?;

    Ok(Page::build(rows, page.page_size(), |b| b.number))
}
