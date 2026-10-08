use sqlx::{AssertSqlSafe, MySql, Pool};

use crate::domains::model::{Page, PageQuery};
use crate::domains::sql::{bind_block_range, block_range_clause};
use crate::errors::error::ApiError;

use super::model::{LogFilter, LogList, LogResponse};

const SELECT_LOG: &str = "SELECT l.`idLogPk` AS page_cursor, \
    l.`blockNumber` AS block_number, b.`timestamp`, l.`logIndex` AS log_index, \
    l.`txHash` AS tx_hash, l.`txIndex` AS tx_index, l.`address`, \
    l.`topic0`, l.`topic1`, l.`topic2`, l.`topic3`, l.`data`, l.`removed` \
    FROM `log` l \
    JOIN `block` b ON b.`number` = l.`blockNumber`";

pub async fn list_logs(
    conn: &Pool<MySql>,
    filter: LogFilter,
    page: PageQuery,
) -> Result<LogList, ApiError> {
    let blocks = block_range_clause("l.`blockNumber`", filter.blocks);

    let mut sql = format!("{SELECT_LOG} WHERE 1 = 1{blocks}");
    if filter.address.is_some() {
        sql.push_str(" AND l.`address` = ?");
    }
    if filter.topic0.is_some() {
        sql.push_str(" AND l.`topic0` = ?");
    }
    if page.cursor().is_some() {
        sql.push_str(" AND l.`idLogPk` < ?");
    }
    sql.push_str(" ORDER BY l.`idLogPk` DESC LIMIT ?");

    let statement = sqlx::query_as::<_, LogResponse>(AssertSqlSafe(sql));
    let mut statement = bind_block_range(statement, filter.blocks);

    if let Some(address) = filter.address.as_deref() {
        statement = statement.bind(address);
    }
    if let Some(topic0) = filter.topic0.as_deref() {
        statement = statement.bind(topic0);
    }
    if let Some(cursor) = page.cursor() {
        statement = statement.bind(cursor);
    }

    let rows = statement.bind(page.fetch_size()).fetch_all(conn).await?;

    Ok(Page::build(rows, page.page_size(), |l| l.page_cursor))
}
