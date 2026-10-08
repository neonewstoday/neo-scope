use sqlx::{MySql, Pool};

use crate::domains::block;
use crate::domains::model::{BlockRange, TimeRange};
use crate::errors::error::ApiError;
use crate::utils::validate::{normalize_address, normalize_hash};

use super::model::{LogFilter, LogList, LogListQuery};
use super::query;

pub async fn list_logs(
    conn: &Pool<MySql>,
    query_params: LogListQuery,
) -> Result<LogList, ApiError> {
    let page = query_params.page();

    let address = query_params
        .address
        .as_deref()
        .map(|value| {
            normalize_address(value).ok_or_else(|| ApiError::bad_request("Invalid address."))
        })
        .transpose()?;

    let topic0 = query_params
        .topic0
        .as_deref()
        .map(|value| normalize_hash(value).ok_or_else(|| ApiError::bad_request("Invalid topic0.")))
        .transpose()?;

    if let (Some(from), Some(to)) = (query_params.from_block, query_params.to_block) {
        if from > to {
            return Err(ApiError::bad_request(
                "`from_block` must not be after `to_block`.",
            ));
        }
    }

    let range = TimeRange::parse(
        query_params.started_at.as_deref(),
        query_params.ended_at.as_deref(),
    )?;
    let blocks = block::query::resolve_block_range(conn, range).await?;
    let blocks = narrow(blocks, query_params.from_block, query_params.to_block);

    query::list_logs(
        conn,
        LogFilter {
            address,
            topic0,
            blocks,
        },
        page,
    )
    .await
}

fn narrow(range: BlockRange, from_block: Option<u64>, to_block: Option<u64>) -> BlockRange {
    BlockRange {
        from: range.from.into_iter().chain(from_block).max(),
        to: range.to.into_iter().chain(to_block).min(),
    }
}
