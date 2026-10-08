use serde::{Deserialize, Serialize};

use crate::domains::model::{Page, PageQuery};

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct TokenResponse {
    #[serde(skip)]
    pub page_cursor: u64,
    pub address: String,
    pub token_type: String,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub decimals: Option<u16>,
    pub first_seen_block: u64,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct TokenTransferResponse {
    #[serde(skip)]
    pub page_cursor: u64,
    pub block_number: u64,
    pub timestamp: u64,
    pub log_index: u32,
    pub sub_index: u32,
    pub tx_hash: String,
    pub tx_index: u32,
    pub token_address: String,
    pub token_type: String,
    pub token_name: Option<String>,
    pub token_symbol: Option<String>,
    pub token_decimals: Option<u16>,
    pub operator: Option<String>,
    pub from_address: String,
    pub to_address: String,
    pub token_id: Option<String>,
    pub amount: String,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct TokenListQuery {
    pub id: Option<u64>,
    pub limit: Option<usize>,
    #[serde(rename = "type")]
    pub token_type: Option<TokenType>,
}

impl TokenListQuery {
    pub fn page(&self) -> PageQuery {
        PageQuery {
            id: self.id,
            limit: self.limit,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum TokenType {
    #[serde(rename = "ERC20", alias = "erc20")]
    Erc20,
    #[serde(rename = "ERC721", alias = "erc721")]
    Erc721,
    #[serde(rename = "ERC1155", alias = "erc1155")]
    Erc1155,
}

impl TokenType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TokenType::Erc20 => "ERC20",
            TokenType::Erc721 => "ERC721",
            TokenType::Erc1155 => "ERC1155",
        }
    }
}

pub type TokenList = Page<TokenResponse>;
pub type TokenTransferList = Page<TokenTransferResponse>;
