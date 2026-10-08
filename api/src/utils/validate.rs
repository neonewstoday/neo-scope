const ADDRESS_HEX_LEN: usize = 40;
const HASH_HEX_LEN: usize = 64;

pub fn normalize_address(value: &str) -> Option<String> {
    normalize_hex(value, ADDRESS_HEX_LEN)
}

pub fn normalize_hash(value: &str) -> Option<String> {
    normalize_hex(value, HASH_HEX_LEN)
}

fn normalize_hex(value: &str, hex_len: usize) -> Option<String> {
    let digits = strip0x(value.trim());

    if digits.len() != hex_len || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }

    Some(format!("0x{}", digits.to_ascii_lowercase()))
}

fn strip0x(value: &str) -> &str {
    value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .unwrap_or(value)
}

#[cfg(test)]
#[path = "tests/validate.rs"]
mod tests;
