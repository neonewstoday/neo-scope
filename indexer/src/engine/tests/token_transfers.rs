use super::*;

const ALICE: &str = "0x000000000000000000000000aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const BOB: &str = "0x000000000000000000000000bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const OPERATOR: &str = "0x000000000000000000000000cccccccccccccccccccccccccccccccccccccccc";

fn log(topics: &[&str], data: &str) -> Log {
    let get = |i: usize| topics.get(i).map(|t: &&str| t.to_string());
    Log {
        block_number: 100,
        log_index: 7,
        tx_hash: "0xtx".to_string(),
        tx_index: 2,
        address: "0xtoken".to_string(),
        topic0: get(0),
        topic1: get(1),
        topic2: get(2),
        topic3: get(3),
        data: data.to_string(),
        removed: false,
    }
}

#[test]
fn decodes_erc20_transfer() {
    let l = log(
        &[TRANSFER, ALICE, BOB],
        "0x0000000000000000000000000000000000000000000000000de0b6b3a7640000",
    );
    let out = decode_transfers(&l);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].token_type, TOKEN_TYPE_ERC20);
    assert_eq!(
        out[0].from_address,
        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    );
    assert_eq!(
        out[0].to_address,
        "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    );
    assert_eq!(out[0].amount, "1000000000000000000");
    assert!(out[0].token_id.is_none());
    assert_eq!(out[0].block_number, 100);
    assert_eq!(out[0].log_index, 7);
}

#[test]
fn decodes_erc721_transfer() {
    let token_id = "0x0000000000000000000000000000000000000000000000000000000000000539";
    let l = log(&[TRANSFER, ALICE, BOB, token_id], "0x");
    let out = decode_transfers(&l);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].token_type, TOKEN_TYPE_ERC721);
    assert_eq!(out[0].token_id.as_deref(), Some("1337"));
    assert_eq!(out[0].amount, "1");
}

#[test]
fn decodes_erc1155_single() {
    let data = "0x0000000000000000000000000000000000000000000000000000000000000002\
                  0000000000000000000000000000000000000000000000000000000000000005";
    let l = log(&[TRANSFER_SINGLE, OPERATOR, ALICE, BOB], data);
    let out = decode_transfers(&l);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].token_type, TOKEN_TYPE_ERC1155);
    assert_eq!(
        out[0].operator.as_deref(),
        Some("0xcccccccccccccccccccccccccccccccccccccccc")
    );
    assert_eq!(out[0].token_id.as_deref(), Some("2"));
    assert_eq!(out[0].amount, "5");
}

#[test]
fn decodes_erc1155_batch_into_one_row_per_entry() {
    let data = "0x0000000000000000000000000000000000000000000000000000000000000040\
                  00000000000000000000000000000000000000000000000000000000000000a0\
                  0000000000000000000000000000000000000000000000000000000000000002\
                  0000000000000000000000000000000000000000000000000000000000000001\
                  0000000000000000000000000000000000000000000000000000000000000002\
                  0000000000000000000000000000000000000000000000000000000000000002\
                  000000000000000000000000000000000000000000000000000000000000000a\
                  0000000000000000000000000000000000000000000000000000000000000014";
    let l = log(&[TRANSFER_BATCH, OPERATOR, ALICE, BOB], data);
    let out = decode_transfers(&l);
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].sub_index, 0);
    assert_eq!(out[0].token_id.as_deref(), Some("1"));
    assert_eq!(out[0].amount, "10");
    assert_eq!(out[1].sub_index, 1);
    assert_eq!(out[1].token_id.as_deref(), Some("2"));
    assert_eq!(out[1].amount, "20");
}

#[test]
fn ignores_unrelated_and_malformed_events() {
    let approval = "0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925";
    assert!(decode_transfers(&log(&[approval, ALICE, BOB], "0x")).is_empty());
    assert!(decode_transfers(&log(&[TRANSFER, ALICE, BOB], "0x")).is_empty());
    let bad = "0x0000000000000000000000000000000000000000000000000000000000000040\
                 0000000000000000000000000000000000000000000000000000000000000080\
                 0000000000000000000000000000000000000000000000000000000000000000\
                 0000000000000000000000000000000000000000000000000000000000000001\
                 000000000000000000000000000000000000000000000000000000000000000a";
    assert!(decode_transfers(&log(&[TRANSFER_BATCH, OPERATOR, ALICE, BOB], bad)).is_empty());
}
