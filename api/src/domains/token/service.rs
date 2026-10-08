use sqlx::{MySql, Pool};

use crate::domains::model::PageQuery;
use crate::errors::error::ApiError;
use crate::utils::validate::normalize_address;

use super::model::{TokenList, TokenListQuery, TokenResponse, TokenTransferList};
use super::query;

pub async fn list_tokens(conn: &Pool<MySql>, query: TokenListQuery) -> Result<TokenList, ApiError> {
    query::list_tokens(conn, query).await
}

pub async fn get_token(conn: &Pool<MySql>, address: String) -> Result<TokenResponse, ApiError> {
    let address = parse_token_address(&address)?;

    query::find_token(conn, &address)
        .await?
        .ok_or_else(|| ApiError::not_found("Token does not exist."))
}

pub async fn list_token_transfers(
    conn: &Pool<MySql>,
    address: String,
    page: PageQuery,
) -> Result<TokenTransferList, ApiError> {
    let address = parse_token_address(&address)?;

    if query::find_token(conn, &address).await?.is_none() {
        return Err(ApiError::not_found("Token does not exist."));
    }

    query::list_transfers_by_token(conn, &address, page).await
}

fn parse_token_address(address: &str) -> Result<String, ApiError> {
    normalize_address(address).ok_or_else(|| ApiError::bad_request("Invalid token address."))
}
