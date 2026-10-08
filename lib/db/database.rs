use anyhow::{Context, Result};
use log::info;
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};
use sqlx::{ConnectOptions, MySql, Pool, Transaction};
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;

use super::migrate::run_migrations;
use super::model::BlockBatch;

pub struct Database {
    pool: Pool<MySql>,
}

fn is_retryable(err: &sqlx::Error) -> bool {
    let sqlx::Error::Database(db_err) = err else {
        return false;
    };
    let code = db_err.code();
    let code = code.as_deref();
    code == Some("1205") || code == Some("1213")
}

async fn sleep_backoff(attempt: usize) {
    let ms = match attempt {
        0 => 50,
        1 => 100,
        2 => 200,
        3 => 400,
        _ => 800,
    };
    sleep(Duration::from_millis(ms)).await;
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        info!("Connecting to database...");

        let connect_opts = MySqlConnectOptions::from_str(database_url)
            .context("Invalid DATABASE_URL")?
            .log_statements(log::LevelFilter::Debug);

        let pool = MySqlPoolOptions::new()
            .connect_with(connect_opts)
            .await
            .context("Failed to open database connection")?;

        run_migrations(&pool)
            .await
            .context("Failed to run database migrations")?;

        Ok(Database { pool })
    }

    pub async fn max_block_number(&self) -> Result<Option<u64>> {
        let result: Option<u64> = sqlx::query_scalar("SELECT MAX(`number`) FROM `block`")
            .fetch_one(&self.pool)
            .await?;
        Ok(result)
    }

    pub async fn known_token_addresses(&self) -> Result<Vec<String>> {
        let rows: Vec<String> = sqlx::query_scalar("SELECT `address` FROM `token`")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    pub async fn insert_batch(&self, batch: &BlockBatch) -> Result<()> {
        let mut attempt = 0usize;
        let max_attempts = 6usize;

        loop {
            let mut tx = self.pool.begin().await?;

            match write_batch(&mut tx, batch).await {
                Ok(()) => {
                    tx.commit().await?;
                    return Ok(());
                }
                Err(e) => {
                    let _ = tx.rollback().await;
                    if is_retryable(&e) && attempt + 1 < max_attempts {
                        attempt += 1;
                        sleep_backoff(attempt - 1).await;
                        continue;
                    }
                    return Err(e.into());
                }
            }
        }
    }
}

async fn write_batch(
    tx: &mut Transaction<'_, MySql>,
    batch: &BlockBatch,
) -> std::result::Result<(), sqlx::Error> {
    for block in &batch.blocks {
        sqlx::query(
            "INSERT IGNORE INTO `block` (
                `number`, `hash`, `parentHash`, `timestamp`, `miner`, `gasUsed`, `gasLimit`,
                `baseFeePerGas`, `size`, `txCount`, `extraData`
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(block.number)
        .bind(&block.hash)
        .bind(&block.parent_hash)
        .bind(block.timestamp)
        .bind(&block.miner)
        .bind(block.gas_used)
        .bind(block.gas_limit)
        .bind(&block.base_fee_per_gas)
        .bind(block.size)
        .bind(block.tx_count)
        .bind(&block.extra_data)
        .execute(&mut **tx)
        .await?;
    }

    for t in &batch.transactions {
        sqlx::query(
            "INSERT IGNORE INTO `transaction` (
                `hash`, `blockNumber`, `txIndex`, `fromAddress`, `toAddress`, `value`,
                `gas`, `gasPrice`, `maxFeePerGas`, `maxPriorityFeePerGas`, `nonce`,
                `txType`, `input`, `status`, `gasUsed`, `cumulativeGasUsed`,
                `effectiveGasPrice`, `contractAddress`
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&t.hash)
        .bind(t.block_number)
        .bind(t.tx_index)
        .bind(&t.from_address)
        .bind(&t.to_address)
        .bind(&t.value)
        .bind(t.gas)
        .bind(&t.gas_price)
        .bind(&t.max_fee_per_gas)
        .bind(&t.max_priority_fee_per_gas)
        .bind(t.nonce)
        .bind(t.tx_type)
        .bind(&t.input)
        .bind(t.status)
        .bind(t.gas_used)
        .bind(t.cumulative_gas_used)
        .bind(&t.effective_gas_price)
        .bind(&t.contract_address)
        .execute(&mut **tx)
        .await?;
    }

    for l in &batch.logs {
        sqlx::query(
            "INSERT IGNORE INTO `log` (
                `blockNumber`, `logIndex`, `txHash`, `txIndex`, `address`,
                `topic0`, `topic1`, `topic2`, `topic3`, `data`, `removed`
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(l.block_number)
        .bind(l.log_index)
        .bind(&l.tx_hash)
        .bind(l.tx_index)
        .bind(&l.address)
        .bind(&l.topic0)
        .bind(&l.topic1)
        .bind(&l.topic2)
        .bind(&l.topic3)
        .bind(&l.data)
        .bind(l.removed)
        .execute(&mut **tx)
        .await?;
    }

    for c in &batch.contracts {
        sqlx::query(
            "INSERT IGNORE INTO `contract` (
                `address`, `creatorAddress`, `creationTxHash`, `creationBlock`,
                `creationTimestamp`, `creationStatus`
            ) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&c.address)
        .bind(&c.creator_address)
        .bind(&c.creation_tx_hash)
        .bind(c.creation_block)
        .bind(c.creation_timestamp)
        .bind(c.creation_status)
        .execute(&mut **tx)
        .await?;
    }

    for a in &batch.addresses {
        sqlx::query(
            "INSERT INTO `address` (`address`, `firstSeenBlock`, `lastSeenBlock`, `isContract`)
             VALUES (?, ?, ?, ?)
             ON DUPLICATE KEY UPDATE
                `firstSeenBlock` = LEAST(`firstSeenBlock`, ?),
                `lastSeenBlock` = GREATEST(`lastSeenBlock`, ?),
                `isContract` = GREATEST(`isContract`, ?)",
        )
        .bind(&a.address)
        .bind(a.first_seen_block)
        .bind(a.last_seen_block)
        .bind(a.is_contract)
        .bind(a.first_seen_block)
        .bind(a.last_seen_block)
        .bind(a.is_contract)
        .execute(&mut **tx)
        .await?;
    }

    for t in &batch.tokens {
        sqlx::query(
            "INSERT INTO `token` (`address`, `tokenType`, `name`, `symbol`, `decimals`, `firstSeenBlock`)
             VALUES (?, ?, ?, ?, ?, ?)
             ON DUPLICATE KEY UPDATE
                `name` = COALESCE(`name`, ?),
                `symbol` = COALESCE(`symbol`, ?),
                `decimals` = COALESCE(`decimals`, ?),
                `firstSeenBlock` = LEAST(`firstSeenBlock`, ?)",
        )
        .bind(&t.address)
        .bind(&t.token_type)
        .bind(&t.name)
        .bind(&t.symbol)
        .bind(t.decimals)
        .bind(t.first_seen_block)
        .bind(&t.name)
        .bind(&t.symbol)
        .bind(t.decimals)
        .bind(t.first_seen_block)
        .execute(&mut **tx)
        .await?;
    }

    for t in &batch.token_transfers {
        sqlx::query(
            "INSERT IGNORE INTO `tokenTransfer` (
                `blockNumber`, `logIndex`, `subIndex`, `txHash`, `txIndex`, `tokenAddress`,
                `tokenType`, `operator`, `fromAddress`, `toAddress`, `tokenId`, `amount`
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(t.block_number)
        .bind(t.log_index)
        .bind(t.sub_index)
        .bind(&t.tx_hash)
        .bind(t.tx_index)
        .bind(&t.token_address)
        .bind(&t.token_type)
        .bind(&t.operator)
        .bind(&t.from_address)
        .bind(&t.to_address)
        .bind(&t.token_id)
        .bind(&t.amount)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}
