use sqlx::{AssertSqlSafe, MySql, Pool};

use crate::domains::model::{BlockRange, Page, PageQuery};
use crate::domains::sql::{bind_block_range, block_range_clause};
use crate::errors::error::ApiError;

use super::model::{ContractList, ContractResponse};

const SELECT_CONTRACT: &str = "SELECT `idContractPk` AS page_cursor, `address`, \
    `creatorAddress` AS creator_address, `creationTxHash` AS creation_tx_hash, \
    `creationBlock` AS creation_block, `creationTimestamp` AS creation_timestamp, \
    `creationStatus` AS creation_status FROM `contract`";

pub async fn find_contract(
    conn: &Pool<MySql>,
    address: &str,
) -> Result<Option<ContractResponse>, ApiError> {
    let sql = format!("{SELECT_CONTRACT} WHERE `address` = ?");

    let contract = sqlx::query_as::<_, ContractResponse>(AssertSqlSafe(sql))
        .bind(address)
        .fetch_optional(conn)
        .await?;

    Ok(contract)
}

pub async fn count_invocations_by_day(
    conn: &Pool<MySql>,
    address: &str,
    range: BlockRange,
) -> Result<Vec<(u64, u64)>, ApiError> {
    let called = block_range_clause("t.`blockNumber`", range);
    let emitted = block_range_clause("l.`blockNumber`", range);

    let sql = format!(
        "SELECT CAST(u.`timestamp` DIV 86400 AS UNSIGNED) AS day_index, \
         CAST(COUNT(*) AS UNSIGNED) AS total FROM ( \
           SELECT t.`hash` AS `txHash`, b.`timestamp` \
             FROM `transaction` t \
             JOIN `block` b ON b.`number` = t.`blockNumber` \
            WHERE t.`toAddress` = ?{called} \
           UNION \
           SELECT l.`txHash`, b.`timestamp` \
             FROM `log` l \
             JOIN `block` b ON b.`number` = l.`blockNumber` \
            WHERE l.`address` = ?{emitted} \
         ) u GROUP BY day_index ORDER BY day_index"
    );

    let mut statement = sqlx::query_as::<_, (u64, u64)>(AssertSqlSafe(sql)).bind(address);
    statement = bind_block_range(statement, range).bind(address);

    let days = bind_block_range(statement, range).fetch_all(conn).await?;

    Ok(days)
}

pub async fn list_contracts(conn: &Pool<MySql>, page: PageQuery) -> Result<ContractList, ApiError> {
    let mut sql = String::from(SELECT_CONTRACT);
    if page.cursor().is_some() {
        sql.push_str(" WHERE `idContractPk` < ?");
    }
    sql.push_str(" ORDER BY `idContractPk` DESC LIMIT ?");

    let mut statement = sqlx::query_as::<_, ContractResponse>(AssertSqlSafe(sql));
    if let Some(cursor) = page.cursor() {
        statement = statement.bind(cursor);
    }

    let rows = statement.bind(page.fetch_size()).fetch_all(conn).await?;

    Ok(Page::build(rows, page.page_size(), |c| c.page_cursor))
}
