use sqlx::{MySql, Pool};

use crate::domains::block;
use crate::domains::model::{PageQuery, TimeRange};
use crate::errors::error::ApiError;
use crate::utils::datetime::day_key;
use crate::utils::validate::normalize_address;

use super::model::{ContractList, ContractResponse, InvocationCountQuery, InvocationCounts};
use super::query;

pub async fn list_contracts(conn: &Pool<MySql>, page: PageQuery) -> Result<ContractList, ApiError> {
    query::list_contracts(conn, page).await
}

pub async fn get_contract(
    conn: &Pool<MySql>,
    address: String,
) -> Result<ContractResponse, ApiError> {
    let address = parse_address(&address)?;

    query::find_contract(conn, &address)
        .await?
        .ok_or_else(|| ApiError::not_found("Contract does not exist."))
}

pub async fn count_contract_invocations(
    conn: &Pool<MySql>,
    address: String,
    query_params: InvocationCountQuery,
) -> Result<InvocationCounts, ApiError> {
    let address = parse_address(&address)?;

    let range = TimeRange::parse(
        query_params.started_at.as_deref(),
        query_params.ended_at.as_deref(),
    )?;
    let range = block::query::resolve_block_range(conn, range).await?;

    let days = query::count_invocations_by_day(conn, &address, range).await?;

    Ok(days
        .into_iter()
        .filter_map(|(day_index, total)| day_key(day_index).map(|key| (key, total)))
        .collect())
}

fn parse_address(address: &str) -> Result<String, ApiError> {
    normalize_address(address).ok_or_else(|| ApiError::bad_request("Invalid contract address."))
}
