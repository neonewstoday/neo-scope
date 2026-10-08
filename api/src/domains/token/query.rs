use sqlx::{AssertSqlSafe, MySql, Pool};

use crate::domains::model::{Page, PageQuery};
use crate::domains::sql::{bind_party_page, party_page_subquery};
use crate::errors::error::ApiError;

use super::model::{
    TokenList, TokenListQuery, TokenResponse, TokenTransferList, TokenTransferResponse,
};

const SELECT_TOKEN: &str = "SELECT `idTokenPk` AS page_cursor, `address`, \
    `tokenType` AS token_type, `name`, `symbol`, `decimals`, \
    `firstSeenBlock` AS first_seen_block FROM `token`";

const SELECT_TOKEN_TRANSFER: &str = "SELECT tt.`idTokenTransferPk` AS page_cursor, \
    tt.`blockNumber` AS block_number, b.`timestamp`, tt.`logIndex` AS log_index, \
    tt.`subIndex` AS sub_index, tt.`txHash` AS tx_hash, tt.`txIndex` AS tx_index, \
    tt.`tokenAddress` AS token_address, tt.`tokenType` AS token_type, \
    tk.`name` AS token_name, tk.`symbol` AS token_symbol, tk.`decimals` AS token_decimals, \
    tt.`operator`, tt.`fromAddress` AS from_address, tt.`toAddress` AS to_address, \
    tt.`tokenId` AS token_id, tt.`amount` \
    FROM `tokenTransfer` tt \
    JOIN `block` b ON b.`number` = tt.`blockNumber` \
    LEFT JOIN `token` tk ON tk.`address` = tt.`tokenAddress`";

pub async fn find_token(
    conn: &Pool<MySql>,
    address: &str,
) -> Result<Option<TokenResponse>, ApiError> {
    let sql = format!("{SELECT_TOKEN} WHERE `address` = ?");

    let token = sqlx::query_as::<_, TokenResponse>(AssertSqlSafe(sql))
        .bind(address)
        .fetch_optional(conn)
        .await?;

    Ok(token)
}

pub async fn list_tokens(conn: &Pool<MySql>, query: TokenListQuery) -> Result<TokenList, ApiError> {
    let page = query.page();

    let mut sql = String::from(SELECT_TOKEN);
    sql.push_str(" WHERE 1 = 1");
    if query.token_type.is_some() {
        sql.push_str(" AND `tokenType` = ?");
    }
    if page.cursor().is_some() {
        sql.push_str(" AND `idTokenPk` < ?");
    }
    sql.push_str(" ORDER BY `idTokenPk` DESC LIMIT ?");

    let mut statement = sqlx::query_as::<_, TokenResponse>(AssertSqlSafe(sql));
    if let Some(token_type) = query.token_type {
        statement = statement.bind(token_type.as_str());
    }
    if let Some(cursor) = page.cursor() {
        statement = statement.bind(cursor);
    }

    let rows = statement.bind(page.fetch_size()).fetch_all(conn).await?;

    Ok(Page::build(rows, page.page_size(), |t| t.page_cursor))
}

pub async fn list_transfers_by_token(
    conn: &Pool<MySql>,
    token_address: &str,
    page: PageQuery,
) -> Result<TokenTransferList, ApiError> {
    let mut sql = format!("{SELECT_TOKEN_TRANSFER} WHERE tt.`tokenAddress` = ?");
    if page.cursor().is_some() {
        sql.push_str(" AND tt.`idTokenTransferPk` < ?");
    }
    sql.push_str(" ORDER BY tt.`idTokenTransferPk` DESC LIMIT ?");

    let mut statement =
        sqlx::query_as::<_, TokenTransferResponse>(AssertSqlSafe(sql)).bind(token_address);
    if let Some(cursor) = page.cursor() {
        statement = statement.bind(cursor);
    }

    let rows = statement.bind(page.fetch_size()).fetch_all(conn).await?;

    Ok(Page::build(rows, page.page_size(), |t| t.page_cursor))
}

pub async fn list_transfers_by_address(
    conn: &Pool<MySql>,
    address: &str,
    page: PageQuery,
) -> Result<TokenTransferList, ApiError> {
    let subquery = party_page_subquery("tokenTransfer", "idTokenTransferPk", page);
    let sql = format!(
        "{SELECT_TOKEN_TRANSFER} \
         JOIN {subquery} page ON page.`idTokenTransferPk` = tt.`idTokenTransferPk` \
         ORDER BY tt.`idTokenTransferPk` DESC"
    );

    let statement = sqlx::query_as::<_, TokenTransferResponse>(AssertSqlSafe(sql));
    let rows = bind_party_page(statement, address, page)
        .fetch_all(conn)
        .await?;

    Ok(Page::build(rows, page.page_size(), |t| t.page_cursor))
}
