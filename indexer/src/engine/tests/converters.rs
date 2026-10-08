use super::*;

const CREATION_RECEIPT: &str = r#"{
    "blockHash": "0xf4292b444fe01e104b834fc468df41a9ae7608ca1525eeec6ffef9c6aedbb86d",
    "blockNumber": "0x4af29",
    "contractAddress": "0x3f05810c438e285dbf81516b44f267358ed94430",
    "cumulativeGasUsed": "0x110b5a",
    "effectiveGasPrice": "0x9502f9000",
    "from": "0x221ba23331e5395f2018edafc2e0e9ff2acb1ada",
    "gasUsed": "0x103ebc",
    "logs": [],
    "logsBloom": "0x00",
    "status": "0x1",
    "to": null,
    "transactionHash": "0x93a54245675d277d02cd3a23f8b18c818cc88a5fe8598a90a765def2541262f7",
    "transactionIndex": "0x1",
    "type": "0x0"
}"#;

const TRANSFER_BATCH_RECEIPT: &str = r#"{
    "blockNumber": "0x4b2e0",
    "contractAddress": null,
    "cumulativeGasUsed": "0x1d155",
    "effectiveGasPrice": "0x9502f9000",
    "from": "0x42869a360781e77ae1f63be4f85e364ef77c66ac",
    "gasUsed": "0x1d155",
    "logs": [{
        "address": "0x8dBDdf1c4bd9a498Ff614d11f3598e29670E43B8",
        "topics": [
            "0x4a39dc06d4c0dbc64b70af90fd698a233a518aa5d07e595d983b8c0526c8f7fb",
            "0x00000000000000000000000042869a360781e77ae1f63be4f85e364ef77c66ac",
            "0x00000000000000000000000042869a360781e77ae1f63be4f85e364ef77c66ac",
            "0x000000000000000000000000d479fdeb2a33fce17298eae59d706e96543648a0"
        ],
        "data": "0x000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001",
        "blockNumber": "0x4b2e0",
        "transactionHash": "0x094dd6a925d54734e70133e88c73fef45cfb1f63f6c122147a2823d5154bf3fc",
        "transactionIndex": "0x0",
        "blockTimestamp": "0x66c4673b",
        "logIndex": "0x0",
        "removed": false
    }],
    "status": "0x1",
    "transactionHash": "0x094dd6a925d54734e70133e88c73fef45cfb1f63f6c122147a2823d5154bf3fc",
    "transactionIndex": "0x0",
    "type": "0x2"
}"#;

fn receipt(json: &str) -> EvmReceipt {
    serde_json::from_str(json).expect("receipt should deserialize")
}

fn block(number: &str, timestamp: &str) -> EvmBlock {
    serde_json::from_str(&format!(
        r#"{{"number":"{number}","hash":"0xaa","parentHash":"0xbb",
             "timestamp":"{timestamp}","miner":"0xCC00000000000000000000000000000000000000",
             "gasUsed":"0x0","gasLimit":"0x0","size":"0x0","transactions":[]}}"#
    ))
    .expect("block should deserialize")
}

#[test]
fn records_top_level_contract_creations() {
    let contracts = convert_contracts(
        &block("0x4af29", "0x66c3fb3e"),
        &[receipt(CREATION_RECEIPT)],
    )
    .unwrap();

    assert_eq!(contracts.len(), 1);
    let c = &contracts[0];
    assert_eq!(c.address, "0x3f05810c438e285dbf81516b44f267358ed94430");
    assert_eq!(
        c.creator_address,
        "0x221ba23331e5395f2018edafc2e0e9ff2acb1ada"
    );
    assert_eq!(c.creation_block, 306985);
    assert_eq!(c.creation_timestamp, 0x66c3_fb3e);
    assert_eq!(c.creation_status, Some(1));
}

#[test]
fn ignores_receipts_without_a_contract_address() {
    let contracts = convert_contracts(
        &block("0x4b2e0", "0x66c4673b"),
        &[receipt(TRANSFER_BATCH_RECEIPT)],
    )
    .unwrap();
    assert!(contracts.is_empty());
}

#[test]
fn flattens_receipt_logs_and_lowercases_addresses() {
    let logs = convert_logs(307936, &[receipt(TRANSFER_BATCH_RECEIPT)]).unwrap();

    assert_eq!(logs.len(), 1);
    let l = &logs[0];
    assert_eq!(l.block_number, 307936);
    assert_eq!(l.log_index, 0);
    assert_eq!(l.tx_index, 0);
    assert_eq!(l.address, "0x8dbddf1c4bd9a498ff614d11f3598e29670e43b8");
    assert_eq!(
        l.topic0.as_deref(),
        Some("0x4a39dc06d4c0dbc64b70af90fd698a233a518aa5d07e595d983b8c0526c8f7fb")
    );
    assert!(l.topic3.is_some());
    assert!(!l.removed);
}

#[test]
fn derives_transfers_and_addresses_end_to_end() {
    let b = block("0x4b2e0", "0x66c4673b");
    let receipts = vec![receipt(TRANSFER_BATCH_RECEIPT)];
    let batch = convert_batch(&[(307936, b)], &HashMap::from([(307936u64, receipts)])).unwrap();

    assert_eq!(batch.logs.len(), 1);
    assert_eq!(batch.token_transfers.len(), 2);
    let ids: Vec<_> = batch
        .token_transfers
        .iter()
        .map(|t| t.token_id.as_deref().unwrap())
        .collect();
    assert_eq!(ids, ["1", "2"]);
    assert!(batch.token_transfers.iter().all(|t| t.amount == "1"));

    let by_addr: HashMap<_, _> = batch
        .addresses
        .iter()
        .map(|a| (a.address.as_str(), a))
        .collect();
    assert!(by_addr["0x8dbddf1c4bd9a498ff614d11f3598e29670e43b8"].is_contract);
    assert!(!by_addr["0xd479fdeb2a33fce17298eae59d706e96543648a0"].is_contract);
    assert!(by_addr.contains_key("0xcc00000000000000000000000000000000000000"));
    assert!(batch
        .addresses
        .iter()
        .all(|a| a.first_seen_block == 307936 && a.last_seen_block == 307936));
}

#[test]
fn collects_addresses_with_the_earliest_block_and_contract_flag() {
    let mut c = AddressCollector::default();
    c.observe("0xabc", 50, false);
    c.observe("0xabc", 10, true);
    c.observe("0xabc", 99, false);

    let rows = c.into_rows();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].first_seen_block, 10);
    assert_eq!(rows[0].last_seen_block, 99);
    assert!(rows[0].is_contract);
}
