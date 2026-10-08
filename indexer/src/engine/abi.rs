use num_bigint::BigUint;

use crate::util::strip0x;

pub type Word = [u8; 32];

pub fn words(data: &str) -> Option<Vec<Word>> {
    let bytes = hex_bytes(data)?;
    if bytes.len() % 32 != 0 {
        return None;
    }
    Some(
        bytes
            .chunks_exact(32)
            .map(|c| {
                let mut w = [0u8; 32];
                w.copy_from_slice(c);
                w
            })
            .collect(),
    )
}

pub fn hex_bytes(s: &str) -> Option<Vec<u8>> {
    let h = strip0x(s.trim());
    if !h.len().is_multiple_of(2) {
        return None;
    }
    (0..h.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&h[i..i + 2], 16).ok())
        .collect()
}

pub fn word_to_address(w: &Word) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(42);
    out.push_str("0x");
    for b in &w[12..] {
        let _ = write!(out, "{:02x}", b);
    }
    out
}

fn be_bytes_to_u64(bytes: &[u8]) -> Option<u64> {
    let (padding, tail) = bytes.split_at(bytes.len().saturating_sub(8));
    if padding.iter().any(|b| *b != 0) {
        return None;
    }
    let mut buf = [0u8; 8];
    buf[8 - tail.len()..].copy_from_slice(tail);
    Some(u64::from_be_bytes(buf))
}

pub fn word_to_decimal(w: &Word) -> String {
    BigUint::from_bytes_be(w).to_string()
}

pub fn topic_to_address(topic: &str) -> Option<String> {
    let bytes = hex_bytes(topic)?;
    if bytes.len() != 32 {
        return None;
    }
    let mut w = [0u8; 32];
    w.copy_from_slice(&bytes);
    Some(word_to_address(&w))
}

pub fn topic_to_decimal(topic: &str) -> Option<String> {
    let bytes = hex_bytes(topic)?;
    if bytes.len() != 32 {
        return None;
    }
    Some(BigUint::from_bytes_be(&bytes).to_string())
}

pub fn decode_string(ret: &str) -> Option<String> {
    let bytes = hex_bytes(ret)?;
    if bytes.is_empty() {
        return None;
    }

    if bytes.len() == 32 {
        let text = clean(&bytes);
        return if text.is_empty() { None } else { Some(text) };
    }

    if bytes.len() < 64 {
        return None;
    }
    let offset = be_bytes_to_u64(&bytes[0..32])? as usize;
    if offset.checked_add(32)? > bytes.len() {
        return None;
    }
    let len = be_bytes_to_u64(&bytes[offset..offset + 32])? as usize;
    let start = offset + 32;
    let end = start.checked_add(len)?;
    if end > bytes.len() {
        return None;
    }
    let text = clean(&bytes[start..end]);
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

pub fn decode_small_uint(ret: &str) -> Option<u16> {
    let bytes = hex_bytes(ret)?;
    if bytes.len() < 32 {
        return None;
    }
    u16::try_from(be_bytes_to_u64(&bytes[0..32])?).ok()
}

fn clean(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .trim()
        .to_string()
}

#[cfg(test)]
#[path = "tests/abi.rs"]
mod tests;
