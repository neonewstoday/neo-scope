use super::*;

const AUGUST_11_2026: i64 = 1_786_406_400;

fn parsed(value: &str, end_of_day: bool) -> i64 {
    parse_timestamp(value, end_of_day).expect("value should parse")
}

#[test]
fn a_date_becomes_the_start_of_its_utc_day() {
    assert_eq!(parsed("2026-08-11", false), AUGUST_11_2026);
}

#[test]
fn a_date_read_as_an_end_becomes_the_last_second_of_its_utc_day() {
    assert_eq!(
        parsed("2026-08-11", true),
        AUGUST_11_2026 + SECONDS_PER_DAY - 1
    );
}

#[test]
fn a_date_ignores_surrounding_whitespace() {
    assert_eq!(parsed("  2026-08-11\n", false), AUGUST_11_2026);
}

#[test]
fn epoch_seconds_pass_through_unchanged() {
    assert_eq!(parsed("1786406400", false), AUGUST_11_2026);
    assert_eq!(parsed("1786406400", true), AUGUST_11_2026);
}

#[test]
fn epoch_milliseconds_are_recognized_and_reduced_to_seconds() {
    assert_eq!(parsed("1786406400000", false), AUGUST_11_2026);
}

#[test]
fn rfc3339_keeps_its_offset() {
    assert_eq!(parsed("2026-08-11T00:00:00Z", false), AUGUST_11_2026);
    assert_eq!(
        parsed("2026-08-11T00:00:00-03:00", false),
        AUGUST_11_2026 + 3 * 3_600
    );
}

#[test]
fn an_unparseable_value_is_a_bad_request() {
    let error = parse_timestamp("yesterday", false).unwrap_err();

    assert!(matches!(error, ApiError::BadRequest(_)));
}

#[test]
fn a_partial_or_impossible_date_is_a_bad_request() {
    assert!(parse_timestamp("2026-08", false).is_err());
    assert!(parse_timestamp("2026-13-01", false).is_err());
}

#[test]
fn a_day_index_formats_as_an_iso_date() {
    assert_eq!(day_key(20_676), Some("2026-08-11".to_string()));
    assert_eq!(day_key(0), Some("1970-01-01".to_string()));
}

#[test]
fn an_out_of_range_day_index_has_no_key() {
    assert_eq!(day_key(u64::MAX), None);
}
