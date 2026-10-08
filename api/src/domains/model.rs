use serde::{Deserialize, Serialize};

use crate::errors::error::ApiError;
use crate::utils::datetime::parse_timestamp;

pub const DEFAULT_PAGE_SIZE: usize = 10;
pub const MAX_PAGE_SIZE: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NextPageParams {
    pub id: u64,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct PageQuery {
    pub id: Option<u64>,
    pub limit: Option<usize>,
}

impl PageQuery {
    pub fn cursor(&self) -> Option<u64> {
        self.id
    }

    pub fn page_size(&self) -> usize {
        self.limit
            .unwrap_or(DEFAULT_PAGE_SIZE)
            .clamp(1, MAX_PAGE_SIZE)
    }

    pub fn fetch_size(&self) -> u64 {
        self.page_size() as u64 + 1
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TimeRange {
    pub started_at: Option<i64>,
    pub ended_at: Option<i64>,
}

impl TimeRange {
    pub fn parse(started_at: Option<&str>, ended_at: Option<&str>) -> Result<Self, ApiError> {
        let started_at = started_at
            .map(|value| parse_timestamp(value, false))
            .transpose()?;
        let ended_at = ended_at
            .map(|value| parse_timestamp(value, true))
            .transpose()?;

        if let (Some(start), Some(end)) = (started_at, ended_at) {
            if start > end {
                return Err(ApiError::bad_request(
                    "`started_at` must not be after `ended_at`.",
                ));
            }
        }

        Ok(TimeRange {
            started_at,
            ended_at,
        })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BlockRange {
    pub from: Option<u64>,
    pub to: Option<u64>,
}

impl BlockRange {
    pub const EMPTY: BlockRange = BlockRange {
        from: Some(1),
        to: Some(0),
    };

    pub fn bounds(&self) -> impl Iterator<Item = u64> + '_ {
        self.from.into_iter().chain(self.to)
    }
}

#[derive(Debug, Serialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_page_params: Option<NextPageParams>,
}

impl<T> Page<T> {
    pub fn build(mut rows: Vec<T>, page_size: usize, cursor_of: impl Fn(&T) -> u64) -> Self {
        let has_next = rows.len() > page_size;
        rows.truncate(page_size);

        let next_page_params = match (has_next, rows.last()) {
            (true, Some(last)) => Some(NextPageParams {
                id: cursor_of(last),
            }),
            _ => None,
        };

        Page {
            items: rows,
            next_page_params,
        }
    }
}

#[cfg(test)]
#[path = "tests/model.rs"]
mod tests;
