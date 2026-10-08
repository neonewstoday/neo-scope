use anyhow::{anyhow, Result};
use num_bigint::BigUint;

pub fn strip0x(s: &str) -> &str {
    s.strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
        .unwrap_or(s)
}

pub fn normalize_address(s: &str) -> String {
    let h = strip0x(s.trim());
    format!("0x{}", h.to_ascii_lowercase())
}

pub fn hex_to_u64(s: &str) -> Result<u64> {
    let h = strip0x(s.trim());
    if h.is_empty() {
        return Ok(0);
    }
    u64::from_str_radix(h, 16).map_err(|e| anyhow!("invalid hex u64 '{}': {}", s, e))
}

pub fn hex_to_u256_decimal(s: &str) -> Result<String> {
    let h = strip0x(s.trim());
    if h.is_empty() {
        return Ok("0".to_string());
    }
    let value = BigUint::parse_bytes(h.as_bytes(), 16)
        .ok_or_else(|| anyhow!("invalid hex u256 '{}'", s))?;
    Ok(value.to_string())
}

pub fn opt_hex_to_u256_decimal(s: &Option<String>) -> Result<Option<String>> {
    match s {
        Some(v) => Ok(Some(hex_to_u256_decimal(v)?)),
        None => Ok(None),
    }
}

pub fn opt_hex_to_u64(s: &Option<String>) -> Result<Option<u64>> {
    match s {
        Some(v) => Ok(Some(hex_to_u64(v)?)),
        None => Ok(None),
    }
}
