use sqlx::{AssertSqlSafe, MySql, Pool};

use crate::domains::model::{Page, PageQuery};
use crate::errors::error::ApiError;

use super::model::{AddressList, AddressResponse};

const SELECT_ADDRESS: &str = "SELECT `idAddressPk` AS page_cursor, `address`, \
    `firstSeenBlock` AS first_seen_block, `lastSeenBlock` AS last_seen_block, \
    `isContract` AS is_contract FROM `address`";

pub async fn find_address(
    conn: &Pool<MySql>,
    address: &str,
) -> Result<Option<AddressResponse>, ApiError> {
    let sql = format!("{SELECT_ADDRESS} WHERE `address` = ?");

    let found = sqlx::query_as::<_, AddressResponse>(AssertSqlSafe(sql))
        .bind(address)
        .fetch_optional(conn)
        .await?;

    Ok(found)
}

pub async fn list_addresses(conn: &Pool<MySql>, page: PageQuery) -> Result<AddressList, ApiError> {
    let mut sql = String::from(SELECT_ADDRESS);
    if page.cursor().is_some() {
        sql.push_str(" WHERE `idAddressPk` < ?");
    }
    sql.push_str(" ORDER BY `idAddressPk` DESC LIMIT ?");

    let mut statement = sqlx::query_as::<_, AddressResponse>(AssertSqlSafe(sql));
    if let Some(cursor) = page.cursor() {
        statement = statement.bind(cursor);
    }

    let rows = statement.bind(page.fetch_size()).fetch_all(conn).await?;

    Ok(Page::build(rows, page.page_size(), |a| a.page_cursor))
}
