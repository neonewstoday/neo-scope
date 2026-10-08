use sqlx::{AssertSqlSafe, MySql, Pool};

use crate::domains::block;
use crate::errors::error::ApiError;

use super::model::{BlockWindow, PeriodCounts, RollingWeekCounts, StatsCounts};
use super::period::{Period, Periods, PERIOD_COUNT};

const SELECT_COUNTS: &str = "SELECT \
    (SELECT CAST(COALESCE(MAX(`number`), 0) AS UNSIGNED) FROM `block`) AS latest_block, \
    (SELECT CAST(COUNT(*) AS UNSIGNED) FROM `block`) AS total_blocks, \
    (SELECT CAST(COUNT(*) AS UNSIGNED) FROM `transaction`) AS total_transactions, \
    (SELECT CAST(COUNT(*) AS UNSIGNED) FROM `log`) AS total_logs, \
    (SELECT CAST(COUNT(*) AS UNSIGNED) FROM `address`) AS total_addresses, \
    (SELECT CAST(COUNT(*) AS UNSIGNED) FROM `contract`) AS total_contracts, \
    (SELECT CAST(COUNT(*) AS UNSIGNED) FROM `token`) AS total_tokens, \
    (SELECT CAST(COUNT(*) AS UNSIGNED) FROM `tokenTransfer`) AS total_token_transfers";

const INVOCATION_SOURCE: &str = "( \
    SELECT t.`hash` AS `txHash`, t.`toAddress` AS `address`, t.`blockNumber` \
      FROM `transaction` t \
      JOIN `address` a ON a.`address` = t.`toAddress` AND a.`isContract` = 1 \
    UNION \
    SELECT l.`txHash`, l.`address`, l.`blockNumber` FROM `log` l \
    ) i";

const PERIOD_NAMES: [&str; PERIOD_COUNT] = [
    "today",
    "yesterday",
    "this_week",
    "last_week",
    "this_month",
    "last_month",
];

pub async fn get_counts(conn: &Pool<MySql>) -> Result<StatsCounts, ApiError> {
    let counts = sqlx::query_as::<_, StatsCounts>(SELECT_COUNTS)
        .fetch_one(conn)
        .await?;

    Ok(counts)
}

pub async fn get_latest_block(conn: &Pool<MySql>) -> Result<u64, ApiError> {
    let latest_block = sqlx::query_scalar::<_, u64>(
        "SELECT CAST(COALESCE(MAX(`number`), 0) AS UNSIGNED) FROM `block`",
    )
    .fetch_one(conn)
    .await?;

    Ok(latest_block)
}

pub async fn count_transactions(conn: &Pool<MySql>) -> Result<u64, ApiError> {
    let total =
        sqlx::query_scalar::<_, u64>("SELECT CAST(COUNT(*) AS UNSIGNED) FROM `transaction`")
            .fetch_one(conn)
            .await?;

    Ok(total)
}

pub async fn resolve_windows(
    conn: &Pool<MySql>,
    periods: &Periods,
) -> Result<[BlockWindow; PERIOD_COUNT], ApiError> {
    let mut windows = [BlockWindow::EMPTY; PERIOD_COUNT];

    for (window, period) in windows.iter_mut().zip(periods.all()) {
        *window = resolve_window(conn, period).await?;
    }

    Ok(windows)
}

pub async fn get_contract_period_counts(
    conn: &Pool<MySql>,
    windows: &[BlockWindow; PERIOD_COUNT],
) -> Result<PeriodCounts, ApiError> {
    period_counts(conn, "`creationBlock`", "`contract`", windows).await
}

pub async fn get_address_period_counts(
    conn: &Pool<MySql>,
    windows: &[BlockWindow; PERIOD_COUNT],
) -> Result<PeriodCounts, ApiError> {
    period_counts(conn, "`firstSeenBlock`", "`address`", windows).await
}

pub async fn get_invocation_period_counts(
    conn: &Pool<MySql>,
    windows: &[BlockWindow; PERIOD_COUNT],
) -> Result<PeriodCounts, ApiError> {
    period_counts(conn, "i.`blockNumber`", INVOCATION_SOURCE, windows).await
}

pub async fn get_rolling_week_counts(
    conn: &Pool<MySql>,
    from_block: Option<u64>,
) -> Result<RollingWeekCounts, ApiError> {
    let Some(from_block) = from_block else {
        return Ok(RollingWeekCounts::default());
    };

    let counts = sqlx::query_as::<_, RollingWeekCounts>(
        "SELECT \
         (SELECT CAST(COUNT(*) AS UNSIGNED) FROM `transaction` WHERE `blockNumber` >= ?) \
             AS transactions, \
         (SELECT CAST(COUNT(*) AS UNSIGNED) FROM `address` WHERE `firstSeenBlock` >= ?) \
             AS addresses, \
         (SELECT CAST(COUNT(*) AS UNSIGNED) FROM `contract` WHERE `creationBlock` >= ?) \
             AS contracts",
    )
    .bind(from_block)
    .bind(from_block)
    .bind(from_block)
    .fetch_one(conn)
    .await?;

    Ok(counts)
}

async fn resolve_window(conn: &Pool<MySql>, period: Period) -> Result<BlockWindow, ApiError> {
    let start = block::query::first_block_at_or_after(conn, period.start).await?;
    let end = block::query::last_block_at_or_before(conn, period.end).await?;

    match (start, end) {
        (Some(start), Some(end)) if start <= end => Ok(BlockWindow { start, end }),
        _ => Ok(BlockWindow::EMPTY),
    }
}

async fn period_counts(
    conn: &Pool<MySql>,
    column: &str,
    source: &str,
    windows: &[BlockWindow; PERIOD_COUNT],
) -> Result<PeriodCounts, ApiError> {
    let buckets = PERIOD_NAMES
        .map(|name| {
            format!(
                "CAST(COALESCE(SUM(CASE WHEN {column} BETWEEN ? AND ? THEN 1 ELSE 0 END), 0) \
                 AS UNSIGNED) AS {name}"
            )
        })
        .join(", ");

    let sql = format!("SELECT CAST(COUNT(*) AS UNSIGNED) AS total, {buckets} FROM {source}");

    let mut statement = sqlx::query_as::<_, PeriodCounts>(AssertSqlSafe(sql));
    for window in windows {
        statement = statement.bind(window.start).bind(window.end);
    }

    let counts = statement.fetch_one(conn).await?;

    Ok(counts)
}
