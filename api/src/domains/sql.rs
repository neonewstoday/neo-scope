use sqlx::mysql::MySqlArguments;
use sqlx::query::QueryAs;
use sqlx::MySql;

use super::model::{BlockRange, PageQuery};

pub fn block_range_clause(column: &str, range: BlockRange) -> String {
    let mut clause = String::new();

    if range.from.is_some() {
        clause.push_str(&format!(" AND {column} >= ?"));
    }
    if range.to.is_some() {
        clause.push_str(&format!(" AND {column} <= ?"));
    }

    clause
}

pub fn bind_block_range<'q, T>(
    mut statement: QueryAs<'q, MySql, T, MySqlArguments>,
    range: BlockRange,
) -> QueryAs<'q, MySql, T, MySqlArguments> {
    for bound in range.bounds() {
        statement = statement.bind(bound);
    }

    statement
}

pub fn party_page_subquery(table: &str, primary_key: &str, page: PageQuery) -> String {
    let cursor = match page.cursor() {
        Some(_) => format!(" AND `{primary_key}` < ?"),
        None => String::new(),
    };

    format!(
        "((SELECT `{primary_key}` FROM `{table}` WHERE `fromAddress` = ?{cursor} \
          ORDER BY `{primary_key}` DESC LIMIT ?) \
         UNION \
         (SELECT `{primary_key}` FROM `{table}` WHERE `toAddress` = ?{cursor} \
          ORDER BY `{primary_key}` DESC LIMIT ?) \
         ORDER BY `{primary_key}` DESC LIMIT ?)"
    )
}

pub fn bind_party_page<'q, T>(
    mut statement: QueryAs<'q, MySql, T, MySqlArguments>,
    address: &'q str,
    page: PageQuery,
) -> QueryAs<'q, MySql, T, MySqlArguments> {
    for _ in 0..2 {
        statement = statement.bind(address);
        if let Some(cursor) = page.cursor() {
            statement = statement.bind(cursor);
        }
        statement = statement.bind(page.fetch_size());
    }

    statement.bind(page.fetch_size())
}
