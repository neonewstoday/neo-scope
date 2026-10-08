use anyhow::Result;
use std::collections::HashMap;

use crate::rpc::model::{EvmBlock, EvmReceipt, EvmTransaction};
use crate::util::{
    hex_to_u256_decimal, hex_to_u64, normalize_address, opt_hex_to_u256_decimal, opt_hex_to_u64,
};
use neo_scope_db::db::model::{Address, Block, BlockBatch, Contract, Log, Transaction};

use super::token_transfers::decode_transfers;

pub fn convert_block(b: &EvmBlock) -> Result<Block> {
    Ok(Block {
        number: hex_to_u64(&b.number)?,
        hash: b.hash.clone(),
        parent_hash: b.parent_hash.clone(),
        timestamp: hex_to_u64(&b.timestamp)?,
        miner: normalize_address(&b.miner),
        gas_used: hex_to_u64(&b.gas_used)?,
        gas_limit: hex_to_u64(&b.gas_limit)?,
        base_fee_per_gas: opt_hex_to_u256_decimal(&b.base_fee_per_gas)?,
        size: hex_to_u64(&b.size)?,
        tx_count: b.transactions.len() as u32,
        extra_data: b.extra_data.clone(),
    })
}

pub fn convert_transactions(b: &EvmBlock, receipts: &[EvmReceipt]) -> Result<Vec<Transaction>> {
    let receipt_by_hash = index_receipts(receipts);
    let block_number = hex_to_u64(&b.number)?;
    let mut out = Vec::with_capacity(b.transactions.len());

    for tx in &b.transactions {
        let receipt = receipt_by_hash.get(tx.hash.as_str());

        let (status, gas_used, cumulative_gas_used, effective_gas_price, contract_address) =
            match receipt {
                Some(r) => (
                    opt_hex_to_u64(&r.status)?.map(|v| v as u8),
                    Some(hex_to_u64(&r.gas_used)?),
                    Some(hex_to_u64(&r.cumulative_gas_used)?),
                    opt_hex_to_u256_decimal(&r.effective_gas_price)?,
                    r.contract_address.as_deref().map(normalize_address),
                ),
                None => (None, None, None, None, None),
            };

        out.push(Transaction {
            hash: tx.hash.clone(),
            block_number,
            tx_index: hex_to_u64(&tx.transaction_index)? as u32,
            from_address: normalize_address(&tx.from),
            to_address: tx.to.as_deref().map(normalize_address),
            value: hex_to_u256_decimal(&tx.value)?,
            gas: hex_to_u64(&tx.gas)?,
            gas_price: opt_hex_to_u256_decimal(&tx.gas_price)?,
            max_fee_per_gas: opt_hex_to_u256_decimal(&tx.max_fee_per_gas)?,
            max_priority_fee_per_gas: opt_hex_to_u256_decimal(&tx.max_priority_fee_per_gas)?,
            nonce: hex_to_u64(&tx.nonce)?,
            tx_type: opt_hex_to_u64(&tx.tx_type)?.unwrap_or(0) as u16,
            input: tx.input.clone(),
            status,
            gas_used,
            cumulative_gas_used,
            effective_gas_price,
            contract_address,
        });
    }

    Ok(out)
}

pub fn convert_logs(block_number: u64, receipts: &[EvmReceipt]) -> Result<Vec<Log>> {
    let mut out = Vec::new();

    for receipt in receipts {
        let receipt_tx_index = opt_hex_to_u64(&receipt.transaction_index)?.unwrap_or(0) as u32;

        for log in &receipt.logs {
            let topic = |i: usize| log.topics.get(i).map(|t| t.to_ascii_lowercase());
            out.push(Log {
                block_number,
                log_index: hex_to_u64(&log.log_index)? as u32,
                tx_hash: log
                    .transaction_hash
                    .clone()
                    .unwrap_or_else(|| receipt.transaction_hash.clone()),
                tx_index: match &log.transaction_index {
                    Some(i) => hex_to_u64(i)? as u32,
                    None => receipt_tx_index,
                },
                address: normalize_address(&log.address),
                topic0: topic(0),
                topic1: topic(1),
                topic2: topic(2),
                topic3: topic(3),
                data: log.data.clone(),
                removed: log.removed,
            });
        }
    }

    Ok(out)
}

pub fn convert_contracts(b: &EvmBlock, receipts: &[EvmReceipt]) -> Result<Vec<Contract>> {
    let block_number = hex_to_u64(&b.number)?;
    let timestamp = hex_to_u64(&b.timestamp)?;
    let tx_by_hash: HashMap<&str, &EvmTransaction> = b
        .transactions
        .iter()
        .map(|t| (t.hash.as_str(), t))
        .collect();

    let mut out = Vec::new();
    for receipt in receipts {
        let Some(address) = receipt.contract_address.as_deref() else {
            continue;
        };

        let creator = receipt
            .from
            .as_deref()
            .or_else(|| {
                tx_by_hash
                    .get(receipt.transaction_hash.as_str())
                    .map(|t| t.from.as_str())
            })
            .map(normalize_address);
        let Some(creator_address) = creator else {
            continue;
        };

        out.push(Contract {
            address: normalize_address(address),
            creator_address,
            creation_tx_hash: receipt.transaction_hash.clone(),
            creation_block: block_number,
            creation_timestamp: timestamp,
            creation_status: opt_hex_to_u64(&receipt.status)?.map(|v| v as u8),
        });
    }

    Ok(out)
}

#[derive(Default)]
struct AddressCollector {
    seen: HashMap<String, Address>,
}

impl AddressCollector {
    fn observe(&mut self, address: &str, block: u64, is_contract: bool) {
        if address.is_empty() {
            return;
        }
        self.seen
            .entry(address.to_string())
            .and_modify(|a| {
                a.first_seen_block = a.first_seen_block.min(block);
                a.last_seen_block = a.last_seen_block.max(block);
                a.is_contract |= is_contract;
            })
            .or_insert_with(|| Address {
                address: address.to_string(),
                first_seen_block: block,
                last_seen_block: block,
                is_contract,
            });
    }

    fn into_rows(self) -> Vec<Address> {
        self.seen.into_values().collect()
    }
}

pub fn convert_batch(
    blocks: &[(u64, EvmBlock)],
    receipts_by_block: &HashMap<u64, Vec<EvmReceipt>>,
) -> Result<BlockBatch> {
    let mut batch = BlockBatch::default();
    let mut addresses = AddressCollector::default();
    let empty: Vec<EvmReceipt> = Vec::new();

    for (number, block) in blocks {
        let receipts = receipts_by_block.get(number).unwrap_or(&empty);

        let converted_block = convert_block(block)?;
        addresses.observe(&converted_block.miner, *number, false);
        batch.blocks.push(converted_block);

        for tx in convert_transactions(block, receipts)? {
            addresses.observe(&tx.from_address, *number, false);
            if let Some(to) = &tx.to_address {
                addresses.observe(to, *number, false);
            }
            batch.transactions.push(tx);
        }

        for contract in convert_contracts(block, receipts)? {
            addresses.observe(&contract.address, *number, true);
            addresses.observe(&contract.creator_address, *number, false);
            batch.contracts.push(contract);
        }

        for log in convert_logs(*number, receipts)? {
            addresses.observe(&log.address, *number, true);

            for transfer in decode_transfers(&log) {
                addresses.observe(&transfer.from_address, *number, false);
                addresses.observe(&transfer.to_address, *number, false);
                if let Some(operator) = &transfer.operator {
                    addresses.observe(operator, *number, false);
                }
                batch.token_transfers.push(transfer);
            }

            batch.logs.push(log);
        }
    }

    batch.addresses = addresses.into_rows();
    Ok(batch)
}

fn index_receipts(receipts: &[EvmReceipt]) -> HashMap<&str, &EvmReceipt> {
    receipts
        .iter()
        .map(|r| (r.transaction_hash.as_str(), r))
        .collect()
}

#[cfg(test)]
#[path = "tests/converters.rs"]
mod tests;
