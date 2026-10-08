use serde::Serialize;

use crate::domains::contract::model::ContractResponse;
use crate::domains::model::Page;
use crate::domains::token::model::TokenResponse;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct AddressResponse {
    #[serde(skip)]
    pub page_cursor: u64,
    pub address: String,
    pub first_seen_block: u64,
    pub last_seen_block: u64,
    pub is_contract: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AddressDetailResponse {
    #[serde(flatten)]
    pub address: AddressResponse,
    pub contract: Option<ContractResponse>,
    pub token: Option<TokenResponse>,
}

pub type AddressList = Page<AddressResponse>;
