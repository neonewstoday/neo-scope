use neo_scope_db::db::model::{Log, TokenTransfer};

use super::abi::{topic_to_address, topic_to_decimal, word_to_address, word_to_decimal, words};

pub const TOKEN_TYPE_ERC20: &str = "ERC20";
pub const TOKEN_TYPE_ERC721: &str = "ERC721";
pub const TOKEN_TYPE_ERC1155: &str = "ERC1155";

const TRANSFER: &str = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";
const TRANSFER_SINGLE: &str = "0xc3d58168c5ae7397731d063d5bbf3d657854427343f4c083240f7aacaa2d0f62";
const TRANSFER_BATCH: &str = "0x4a39dc06d4c0dbc64b70af90fd698a233a518aa5d07e595d983b8c0526c8f7fb";

pub fn decode_transfers(log: &Log) -> Vec<TokenTransfer> {
    let topic0 = match log.topic0.as_deref() {
        Some(t) => t.to_ascii_lowercase(),
        None => return Vec::new(),
    };

    match topic0.as_str() {
        TRANSFER => decode_transfer(log),
        TRANSFER_SINGLE => decode_transfer_single(log),
        TRANSFER_BATCH => decode_transfer_batch(log),
        _ => Vec::new(),
    }
}

fn decode_transfer(log: &Log) -> Vec<TokenTransfer> {
    if let (Some(t1), Some(t2), Some(t3)) = (&log.topic1, &log.topic2, &log.topic3) {
        let (Some(from), Some(to), Some(token_id)) = (
            topic_to_address(t1),
            topic_to_address(t2),
            topic_to_decimal(t3),
        ) else {
            return Vec::new();
        };
        return vec![transfer(
            log,
            0,
            TOKEN_TYPE_ERC721,
            None,
            from,
            to,
            Some(token_id),
            "1".to_string(),
        )];
    }

    if let (Some(t1), Some(t2)) = (&log.topic1, &log.topic2) {
        let Some(data) = words(&log.data) else {
            return Vec::new();
        };
        let (Some(from), Some(to), Some(amount)) = (
            topic_to_address(t1),
            topic_to_address(t2),
            data.first().map(word_to_decimal),
        ) else {
            return Vec::new();
        };
        return vec![transfer(
            log,
            0,
            TOKEN_TYPE_ERC20,
            None,
            from,
            to,
            None,
            amount,
        )];
    }

    if log.topic1.is_none() {
        let Some(data) = words(&log.data) else {
            return Vec::new();
        };
        if data.len() >= 3 {
            return vec![transfer(
                log,
                0,
                TOKEN_TYPE_ERC20,
                None,
                word_to_address(&data[0]),
                word_to_address(&data[1]),
                None,
                word_to_decimal(&data[2]),
            )];
        }
    }

    Vec::new()
}

fn decode_transfer_single(log: &Log) -> Vec<TokenTransfer> {
    let (Some(t1), Some(t2), Some(t3)) = (&log.topic1, &log.topic2, &log.topic3) else {
        return Vec::new();
    };
    let Some(data) = words(&log.data) else {
        return Vec::new();
    };
    if data.len() < 2 {
        return Vec::new();
    }
    let (Some(operator), Some(from), Some(to)) = (
        topic_to_address(t1),
        topic_to_address(t2),
        topic_to_address(t3),
    ) else {
        return Vec::new();
    };

    vec![transfer(
        log,
        0,
        TOKEN_TYPE_ERC1155,
        Some(operator),
        from,
        to,
        Some(word_to_decimal(&data[0])),
        word_to_decimal(&data[1]),
    )]
}

fn decode_transfer_batch(log: &Log) -> Vec<TokenTransfer> {
    let (Some(t1), Some(t2), Some(t3)) = (&log.topic1, &log.topic2, &log.topic3) else {
        return Vec::new();
    };
    let Some(data) = words(&log.data) else {
        return Vec::new();
    };
    if data.len() < 2 {
        return Vec::new();
    }
    let (Some(operator), Some(from), Some(to)) = (
        topic_to_address(t1),
        topic_to_address(t2),
        topic_to_address(t3),
    ) else {
        return Vec::new();
    };

    let (Some(ids), Some(values)) = (read_uint_array(&data, 0), read_uint_array(&data, 1)) else {
        return Vec::new();
    };
    if ids.len() != values.len() {
        return Vec::new();
    }

    ids.into_iter()
        .zip(values)
        .enumerate()
        .map(|(i, (id, value))| {
            transfer(
                log,
                i as u32,
                TOKEN_TYPE_ERC1155,
                Some(operator.clone()),
                from.clone(),
                to.clone(),
                Some(id),
                value,
            )
        })
        .collect()
}

fn read_uint_array(data: &[[u8; 32]], slot: usize) -> Option<Vec<String>> {
    let offset_bytes = to_index(data.get(slot)?)?;
    if offset_bytes % 32 != 0 {
        return None;
    }
    let head = offset_bytes / 32;
    let len = to_index(data.get(head)?)?;
    let start = head.checked_add(1)?;
    let end = start.checked_add(len)?;
    if end > data.len() {
        return None;
    }
    Some(data[start..end].iter().map(word_to_decimal).collect())
}

fn to_index(w: &[u8; 32]) -> Option<usize> {
    if w[..24].iter().any(|b| *b != 0) {
        return None;
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&w[24..]);
    usize::try_from(u64::from_be_bytes(buf)).ok()
}

#[allow(clippy::too_many_arguments)]
fn transfer(
    log: &Log,
    sub_index: u32,
    token_type: &str,
    operator: Option<String>,
    from_address: String,
    to_address: String,
    token_id: Option<String>,
    amount: String,
) -> TokenTransfer {
    TokenTransfer {
        block_number: log.block_number,
        log_index: log.log_index,
        sub_index,
        tx_hash: log.tx_hash.clone(),
        tx_index: log.tx_index,
        token_address: log.address.clone(),
        token_type: token_type.to_string(),
        operator,
        from_address,
        to_address,
        token_id,
        amount,
    }
}

#[cfg(test)]
#[path = "tests/token_transfers.rs"]
mod tests;
