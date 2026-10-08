use super::*;

const AUGUST_11_2026: i64 = 1_786_406_400;
const NOON: i64 = 12 * 3_600;

fn periods_at(seconds: i64) -> Periods {
    let now = DateTime::from_timestamp(seconds, 0).expect("timestamp should be in range");

    Periods::at(now)
}

fn tuesday_noon() -> Periods {
    periods_at(AUGUST_11_2026 + NOON)
}

#[test]
fn today_starts_at_midnight_and_ends_now() {
    let periods = tuesday_noon();

    assert_eq!(periods.today.start, AUGUST_11_2026);
    assert_eq!(periods.today.end, AUGUST_11_2026 + NOON);
}

#[test]
fn yesterday_is_a_whole_utc_day() {
    let periods = tuesday_noon();

    assert_eq!(periods.yesterday.start, AUGUST_11_2026 - SECONDS_PER_DAY);
    assert_eq!(periods.yesterday.end, AUGUST_11_2026 - 1);
}

#[test]
fn this_week_starts_on_monday() {
    let periods = tuesday_noon();

    assert_eq!(periods.this_week.start, AUGUST_11_2026 - SECONDS_PER_DAY);
    assert_eq!(periods.this_week.end, AUGUST_11_2026 + NOON);
}

#[test]
fn a_monday_starts_its_own_week() {
    let periods = periods_at(AUGUST_11_2026 - SECONDS_PER_DAY + NOON);

    assert_eq!(periods.this_week.start, AUGUST_11_2026 - SECONDS_PER_DAY);
}

#[test]
fn a_sunday_still_belongs_to_the_week_that_started_on_monday() {
    let periods = periods_at(AUGUST_11_2026 + 5 * SECONDS_PER_DAY + NOON);

    assert_eq!(periods.this_week.start, AUGUST_11_2026 - SECONDS_PER_DAY);
}

#[test]
fn last_week_is_the_seven_days_before_this_one() {
    let periods = tuesday_noon();

    assert_eq!(
        periods.last_week.start,
        periods.this_week.start - 7 * SECONDS_PER_DAY
    );
    assert_eq!(periods.last_week.end, periods.this_week.start - 1);
}

#[test]
fn this_month_starts_on_the_first() {
    let periods = tuesday_noon();

    assert_eq!(
        periods.this_month.start,
        AUGUST_11_2026 - 10 * SECONDS_PER_DAY
    );
    assert_eq!(periods.this_month.end, AUGUST_11_2026 + NOON);
}

#[test]
fn last_month_ends_the_second_before_this_one_starts() {
    let periods = tuesday_noon();

    assert_eq!(periods.last_month.end, periods.this_month.start - 1);
    assert_eq!(
        periods.last_month.start,
        periods.this_month.start - 31 * SECONDS_PER_DAY
    );
}

#[test]
fn last_month_follows_a_short_february() {
    let march_1_2026 = AUGUST_11_2026 - 163 * SECONDS_PER_DAY;
    let periods = periods_at(march_1_2026 + NOON);

    assert_eq!(periods.this_month.start, march_1_2026);
    assert_eq!(
        periods.last_month.start,
        march_1_2026 - 28 * SECONDS_PER_DAY
    );
    assert_eq!(periods.last_month.end, march_1_2026 - 1);
}

#[test]
fn last_month_crosses_the_year_boundary() {
    let january_15_2026 = AUGUST_11_2026 - 208 * SECONDS_PER_DAY;
    let periods = periods_at(january_15_2026 + NOON);

    assert_eq!(
        periods.this_month.start,
        january_15_2026 - 14 * SECONDS_PER_DAY
    );
    assert_eq!(
        periods.last_month.start,
        periods.this_month.start - 31 * SECONDS_PER_DAY
    );
}

#[test]
fn the_rolling_week_ends_now_and_spans_seven_days() {
    let periods = tuesday_noon();

    assert_eq!(
        periods.rolling_week_start,
        AUGUST_11_2026 + NOON - 7 * SECONDS_PER_DAY
    );
}

#[test]
fn every_period_is_listed_in_response_order() {
    let periods = tuesday_noon();

    assert_eq!(
        periods.all(),
        [
            periods.today,
            periods.yesterday,
            periods.this_week,
            periods.last_week,
            periods.this_month,
            periods.last_month,
        ]
    );
}
