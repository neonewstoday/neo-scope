use chrono::{DateTime, NaiveDate, NaiveTime, Utc};

use crate::errors::error::ApiError;

pub const SECONDS_PER_DAY: i64 = 86_400;

const MILLISECOND_THRESHOLD: i64 = 100_000_000_000;

pub fn now_timestamp_ms() -> i64 {
    Utc::now().timestamp_millis()
}

pub fn parse_timestamp(value: &str, end_of_day: bool) -> Result<i64, ApiError> {
    let value = value.trim();

    if let Ok(number) = value.parse::<i64>() {
        return Ok(if number.abs() >= MILLISECOND_THRESHOLD {
            number.div_euclid(1_000)
        } else {
            number
        });
    }

    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        let start = date.and_time(NaiveTime::MIN).and_utc().timestamp();

        return Ok(if end_of_day {
            start + SECONDS_PER_DAY - 1
        } else {
            start
        });
    }

    if let Ok(datetime) = DateTime::parse_from_rfc3339(value) {
        return Ok(datetime.timestamp());
    }

    Err(ApiError::bad_request(format!(
        "Invalid datetime `{value}`. Use epoch seconds, `YYYY-MM-DD` or RFC 3339."
    )))
}

pub fn day_key(day_index: u64) -> Option<String> {
    let seconds = i64::try_from(day_index)
        .ok()?
        .checked_mul(SECONDS_PER_DAY)?;

    DateTime::from_timestamp(seconds, 0).map(|at| at.format("%Y-%m-%d").to_string())
}

#[cfg(test)]
#[path = "tests/datetime.rs"]
mod tests;
