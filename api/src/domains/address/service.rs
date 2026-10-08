use sqlx::{MySql, Pool};

use crate::domains::contract;
use crate::domains::model::PageQuery;
use crate::domains::token::{self, model::TokenTransferList};
use crate::domains::transaction::{self, model::TransactionList};
use crate::errors::error::ApiError;
use crate::utils::validate::normalize_address;

use super::model::{AddressDetailResponse, AddressList, AddressResponse};
use super::query;

pub async fn list_addresses(conn: &Pool<MySql>, page: PageQuery) -> Result<AddressList, ApiError> {
    query::list_addresses(conn, page).await
}

pub async fn get_address(
    conn: &Pool<MySql>,
    address: String,
) -> Result<AddressDetailResponse, ApiError> {
    let address = parse_address(&address)?;
    let found = require_address(conn, &address).await?;

    let (contract, token) = tokio::join!(
        contract::query::find_contract(conn, &address),
        token::query::find_token(conn, &address)
    );

    Ok(AddressDetailResponse {
        address: found,
        contract: contract?,
        token: token?,
    })
}

pub async fn list_address_transactions(
    conn: &Pool<MySql>,
    address: String,
    page: PageQuery,
) -> Result<TransactionList, ApiError> {
    let address = parse_address(&address)?;
    require_address(conn, &address).await?;

    transaction::query::list_transactions_by_address(conn, &address, page).await
}

pub async fn list_address_transfers(
    conn: &Pool<MySql>,
    address: String,
    page: PageQuery,
) -> Result<TokenTransferList, ApiError> {
    let address = parse_address(&address)?;
    require_address(conn, &address).await?;

    token::query::list_transfers_by_address(conn, &address, page).await
}

async fn require_address(conn: &Pool<MySql>, address: &str) -> Result<AddressResponse, ApiError> {
    query::find_address(conn, address)
        .await?
        .ok_or_else(|| ApiError::not_found("Address does not exist."))
}

fn parse_address(address: &str) -> Result<String, ApiError> {
    normalize_address(address).ok_or_else(|| ApiError::bad_request("Invalid address."))
}
