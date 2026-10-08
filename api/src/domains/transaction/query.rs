use sqlx::{AssertSqlSafe, MySql, Pool};

use crate::domains::model::{Page, PageQuery};
use crate::domains::sql::{bind_party_page, party_page_subquery};
use crate::errors::error::ApiError;

use super::model::{TransactionList, TransactionResponse};

const SELECT_TRANSACTION: &str = "SELECT t.`idTransactionPk` AS page_cursor, t.`hash`, \
    t.`blockNumber` AS block_number, b.`timestamp`, t.`txIndex` AS tx_index, \
    t.`fromAddress` AS from_address, t.`toAddress` AS to_address, t.`value`, t.`gas`, \
    t.`gasPrice` AS gas_price, t.`maxFeePerGas` AS max_fee_per_gas, \
    t.`maxPriorityFeePerGas` AS max_priority_fee_per_gas, t.`nonce`, t.`txType` AS tx_type, \
    t.`input`, t.`status`, t.`gasUsed` AS gas_used, \
    t.`cumulativeGasUsed` AS cumulative_gas_used, \
    t.`effectiveGasPrice` AS effective_gas_price, t.`contractAddress` AS contract_address \
    FROM `transaction` t JOIN `block` b ON b.`number` = t.`blockNumber`";

pub async fn find_transaction_by_hash(
    conn: &Pool<MySql>,
    hash: &str,
) -> Result<Option<TransactionResponse>, ApiError> {
    let sql = format!("{SELECT_TRANSACTION} WHERE t.`hash` = ?");

    let transaction = sqlx::query_as::<_, TransactionResponse>(AssertSqlSafe(sql))
        .bind(hash)
        .fetch_optional(conn)
        .await?;

    Ok(transaction)
}

pub async fn list_transactions(
    conn: &Pool<MySql>,
    page: PageQuery,
) -> Result<TransactionList, ApiError> {
    let mut sql = String::from(SELECT_TRANSACTION);
    if page.cursor().is_some() {
        sql.push_str(" WHERE t.`idTransactionPk` < ?");
    }
    sql.push_str(" ORDER BY t.`idTransactionPk` DESC LIMIT ?");

    let mut query = sqlx::query_as::<_, TransactionResponse>(AssertSqlSafe(sql));
    if let Some(cursor) = page.cursor() {
        query = query.bind(cursor);
    }

    let rows = query.bind(page.fetch_size()).fetch_all(conn).await?;

    Ok(Page::build(rows, page.page_size(), |t| t.page_cursor))
}

pub async fn list_transactions_by_block(
    conn: &Pool<MySql>,
    block_number: u64,
    page: PageQuery,
) -> Result<TransactionList, ApiError> {
    let mut sql = format!("{SELECT_TRANSACTION} WHERE t.`blockNumber` = ?");
    if page.cursor().is_some() {
        sql.push_str(" AND t.`idTransactionPk` < ?");
    }
    sql.push_str(" ORDER BY t.`idTransactionPk` DESC LIMIT ?");

    let mut query = sqlx::query_as::<_, TransactionResponse>(AssertSqlSafe(sql)).bind(block_number);
    if let Some(cursor) = page.cursor() {
        query = query.bind(cursor);
    }

    let rows = query.bind(page.fetch_size()).fetch_all(conn).await?;

    Ok(Page::build(rows, page.page_size(), |t| t.page_cursor))
}

pub async fn list_transactions_by_address(
    conn: &Pool<MySql>,
    address: &str,
    page: PageQuery,
) -> Result<TransactionList, ApiError> {
    let subquery = party_page_subquery("transaction", "idTransactionPk", page);
    let sql = format!(
        "{SELECT_TRANSACTION} \
         JOIN {subquery} page ON page.`idTransactionPk` = t.`idTransactionPk` \
         ORDER BY t.`idTransactionPk` DESC"
    );

    let statement = sqlx::query_as::<_, TransactionResponse>(AssertSqlSafe(sql));
    let rows = bind_party_page(statement, address, page)
        .fetch_all(conn)
        .await?;

    Ok(Page::build(rows, page.page_size(), |t| t.page_cursor))
}
