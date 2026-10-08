use super::*;

const ADDRESS: &str = "0x1234567890abcdef1234567890abcdef12345678";
const HASH: &str = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

#[test]
fn address_keeps_a_lowercase_prefixed_value() {
    assert_eq!(normalize_address(ADDRESS), Some(ADDRESS.to_string()));
}

#[test]
fn address_lowercases_a_checksummed_value() {
    let checksummed = "0xAbC4567890AbCdEf1234567890aBcDeF12345678";

    assert_eq!(
        normalize_address(checksummed),
        Some("0xabc4567890abcdef1234567890abcdef12345678".to_string())
    );
}

#[test]
fn address_accepts_a_value_without_the_prefix() {
    let bare = &ADDRESS[2..];

    assert_eq!(normalize_address(bare), Some(ADDRESS.to_string()));
}

#[test]
fn address_accepts_an_uppercase_prefix() {
    let prefixed = format!("0X{}", &ADDRESS[2..]);

    assert_eq!(normalize_address(&prefixed), Some(ADDRESS.to_string()));
}

#[test]
fn address_trims_surrounding_whitespace() {
    let padded = format!("  {ADDRESS}\n");

    assert_eq!(normalize_address(&padded), Some(ADDRESS.to_string()));
}

#[test]
fn address_rejects_a_wrong_length() {
    assert_eq!(normalize_address(&ADDRESS[..41]), None);
    assert_eq!(normalize_address(&format!("{ADDRESS}0")), None);
}

#[test]
fn address_rejects_non_hex_digits() {
    let invalid = "0x1234567890abcdef1234567890abcdef1234567z";

    assert_eq!(normalize_address(invalid), None);
}

#[test]
fn address_rejects_an_empty_value() {
    assert_eq!(normalize_address(""), None);
    assert_eq!(normalize_address("0x"), None);
}

#[test]
fn address_rejects_a_hash() {
    assert_eq!(normalize_address(HASH), None);
}

#[test]
fn hash_keeps_a_lowercase_prefixed_value() {
    assert_eq!(normalize_hash(HASH), Some(HASH.to_string()));
}

#[test]
fn hash_lowercases_and_accepts_a_bare_value() {
    let bare_upper = HASH[2..].to_ascii_uppercase();

    assert_eq!(normalize_hash(&bare_upper), Some(HASH.to_string()));
}

#[test]
fn hash_rejects_an_address() {
    assert_eq!(normalize_hash(ADDRESS), None);
}

#[test]
fn hash_rejects_non_hex_digits() {
    let invalid = format!("0x{}g", &HASH[2..65]);

    assert_eq!(normalize_hash(&invalid), None);
}
