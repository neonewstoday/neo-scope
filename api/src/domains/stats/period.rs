use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, Utc};

use crate::utils::datetime::SECONDS_PER_DAY;

pub const PERIOD_COUNT: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Period {
    pub start: i64,
    pub end: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Periods {
    pub today: Period,
    pub yesterday: Period,
    pub this_week: Period,
    pub last_week: Period,
    pub this_month: Period,
    pub last_month: Period,
    pub rolling_week_start: i64,
}

impl Periods {
    pub fn at(now: DateTime<Utc>) -> Self {
        let now_seconds = now.timestamp();
        let today = now.date_naive();
        let yesterday = today - Duration::days(1);

        let this_week_start = week_start(today);
        let last_week_start = this_week_start - Duration::days(7);
        let last_week_end = this_week_start - Duration::days(1);

        let this_month_start = month_start(today);
        let last_month_end = this_month_start - Duration::days(1);
        let last_month_start = month_start(last_month_end);

        Periods {
            today: Period {
                start: day_start(today),
                end: now_seconds,
            },
            yesterday: Period {
                start: day_start(yesterday),
                end: day_end(yesterday),
            },
            this_week: Period {
                start: day_start(this_week_start),
                end: now_seconds,
            },
            last_week: Period {
                start: day_start(last_week_start),
                end: day_end(last_week_end),
            },
            this_month: Period {
                start: day_start(this_month_start),
                end: now_seconds,
            },
            last_month: Period {
                start: day_start(last_month_start),
                end: day_end(last_month_end),
            },
            rolling_week_start: now_seconds - 7 * SECONDS_PER_DAY,
        }
    }

    pub fn all(&self) -> [Period; PERIOD_COUNT] {
        [
            self.today,
            self.yesterday,
            self.this_week,
            self.last_week,
            self.this_month,
            self.last_month,
        ]
    }
}

fn day_start(date: NaiveDate) -> i64 {
    date.and_time(NaiveTime::MIN).and_utc().timestamp()
}

fn day_end(date: NaiveDate) -> i64 {
    day_start(date) + SECONDS_PER_DAY - 1
}

fn week_start(date: NaiveDate) -> NaiveDate {
    date - Duration::days(i64::from(date.weekday().num_days_from_monday()))
}

fn month_start(date: NaiveDate) -> NaiveDate {
    date.with_day(1).unwrap_or(date)
}

#[cfg(test)]
#[path = "tests/period.rs"]
mod tests;
