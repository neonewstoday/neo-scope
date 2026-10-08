# neo-scope

[![CI](https://github.com/neonewstoday/neo-scope/actions/workflows/ci.yml/badge.svg)](https://github.com/neonewstoday/neo-scope/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Rust 1.94+](https://img.shields.io/badge/rust-1.94%2B-orange.svg)](https://www.rust-lang.org)
[![Status: alpha](https://img.shields.io/badge/status-alpha-yellow.svg)](#status)

An open source indexer and read-only HTTP API for the **Neo X** EVM sidechain, written in Rust.

neo-scope syncs blocks, transactions, receipts, logs, contracts, addresses, tokens and token
transfers from a Neo X JSON-RPC node into MySQL, and serves them over a REST API built for
block explorers and analytics. It targets Neo X only. Unlike Neo N3, Neo X uses
Ethereum-style RPC (`eth_getBlockByNumber`, `eth_getBlockReceipts`) and a completely different
data model.

> **Community project.** neo-scope is maintained by [Neo News Today](https://neonewstoday.com)
> and is not affiliated with or endorsed by the Neo Foundation or Neo Global Development.
> "Neo" is a trademark of its respective owner.

## Status

**Alpha.** The indexer and API are functional, but the database schema and the API responses
may still change without a migration path between releases. Pin a
version if you depend on it, and expect to re-index after upgrading.

## Quickstart

### With Docker

Runs MySQL 8, the indexer and the API against testnet T4. Only Docker is required.

```bash
git clone https://github.com/neonewstoday/neo-scope.git
cd neo-scope
docker compose up -d --build
curl "http://localhost:8083/v1/neox/block?limit=5"
```

The indexer starts at genesis by default. To start near the chain tip instead, set
`START_BLOCK` before the first run (it only applies to an empty database):

```bash
START_BLOCK=10000000 docker compose up -d --build
```

`NODE_URL`, `CHAIN_ID`, `BATCH_SIZE`, `API_PORT` and `RUST_LOG` can be overridden the same
way. Follow the indexer with `docker compose logs -f indexer`; `docker compose down -v`
stops everything and deletes the database.

### From source

You need Rust 1.94+ and a MySQL 8 database; see [Prerequisites](#prerequisites).

```bash
cp .env.example .env            # points at testnet T4 and a local MySQL
mysql -u root -p -e "CREATE DATABASE neox_testnet"
cargo run --release --package neo-scope-indexer
```

In a second terminal, once the indexer has written its first blocks:

```bash
cargo run --release --package neo-scope-api
curl http://localhost:8083/v1/neox/stats
```

## What it indexes

Core:

- **block:** number, hash, parent hash, timestamp, miner, gas used/limit, base fee, size, tx count.
- **transaction:** hash, block, index, from/to, value (wei), gas, gas price / EIP-1559 fees,
  nonce, type, input, plus receipt-derived `status`, `gasUsed`, `cumulativeGasUsed`,
  `effectiveGasPrice` and `contractAddress`.

Derived from the same receipts, with no extra RPC round trips except the metadata reads
described under **token**:

- **log:** every event log, uniquely keyed by `(blockNumber, logIndex)`. Topics are
  flattened into `topic0`..`topic3` so they can be indexed and filtered directly.
- **contract:** top-level contract creations (from the receipt's `contractAddress`), with
  creator, creation tx/block/timestamp and whether the creating tx succeeded. Contracts
  deployed from inside another contract require tracing and are not covered.
- **address:** every address observed in any role (sender, recipient, miner, log emitter,
  token transfer party, deployed contract), with `firstSeenBlock`, `lastSeenBlock` and an
  `isContract` flag. `firstSeenBlock` is kept as the minimum over all sightings, so
  backfilling an older range corrects it.
- **token:** token contracts discovered from the transfer events they emit, with
  `tokenType` (`ERC20` / `ERC721` / `ERC1155`) and `name` / `symbol` / `decimals`. Metadata
  is read once per token via `eth_call`: two calls (`name`, `symbol`) for ERC-721 and
  ERC-1155, three for ERC-20 (plus `decimals`). A getter that reverts or is absent is not an
  error: the column is left NULL and is never retried.
- **tokenTransfer:** decoded transfers. `Transfer(address,address,uint256)` is read as
  ERC-721 when the token id is indexed (four topics) and as ERC-20 otherwise; ERC-1155
  `TransferSingle` and `TransferBatch` are decoded too, with a batch producing one row per
  id/value pair (`subIndex`).

Amounts and token ids are stored as decimal strings, the same way wei values are.

### Not covered

Internal transactions and contracts deployed from inside another contract (both need
`debug_traceTransaction`), token balances, and pending transactions.

### Case normalization

Addresses are normalized to lowercase everywhere they are written: `miner`, `fromAddress`,
`toAddress`, `contractAddress`, a log's emitter, and every address decoded out of a transfer.
Log topics are lowercased too.

Hashes (`hash`, `parentHash`, `txHash`, `creationTxHash`) are stored exactly as the node
returns them. Geth-family nodes return them lowercase, so the tables join cleanly in
practice, but the indexer does not enforce it. Only addresses use EIP-55 checksumming, which
is what the normalization exists for.

## What it serves

The `api` crate exposes the tables above over HTTP under `/v1/neox`, with cursor pagination
through `next_page_params` and a uniform `{"error": …}` body:

```
GET /v1/neox/block                              GET /v1/neox/contract
GET /v1/neox/block/{number|hash}                GET /v1/neox/contract/{address}
GET /v1/neox/block/{number|hash}/transactions   GET /v1/neox/contract/{address}/invocations
GET /v1/neox/transaction                        GET /v1/neox/token
GET /v1/neox/transaction/{hash}                 GET /v1/neox/token/{address}
GET /v1/neox/address                            GET /v1/neox/token/{address}/transfers
GET /v1/neox/address/{address}                  GET /v1/neox/log
GET /v1/neox/address/{address}/transactions     GET /v1/neox/stats
GET /v1/neox/address/{address}/transfers        GET /v1/neox/stat/neox-statistics
                                                GET /v1/neox/stat/network-statistics
```

`/contract/{address}/invocations` returns a `{"YYYY-MM-DD": count}` map. A transaction counts
as an invocation of a contract when the contract is the transaction's recipient **or** the
emitter of one of its logs, which is what keeps calls arriving through a router or proxy from
disappearing. The two `/stat/` endpoints are per-period aggregates (today, yesterday, this/last week, this/last month).

Wei and other u256 values stay decimal strings on the way out, exactly as stored. Full
reference (pagination, types, error codes, layout) is in the [api README](api/README.md).

## Prerequisites

- Rust 1.94 or newer (required by sqlx 0.9). Developed on 1.97.
- MySQL 8 with InnoDB (the schema uses foreign keys), and an empty database.
- A Neo X JSON-RPC endpoint.

## Configure

Copy `.env.example` to `.env` and adjust.

| Variable              | Default    | Meaning                                                                                                                                  |
| --------------------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| `DATABASE_URL`        | _required_ | MySQL DSN, e.g. `mysql://user:pass@localhost:3306/neox`.                                                                                 |
| `NODE_URL`            | _required_ | Neo X JSON-RPC endpoint.                                                                                                                 |
| `CHAIN_ID`            | unset      | When set, the node's `eth_chainId` must match or startup fails.                                                                          |
| `BATCH_SIZE`          | `25`       | Blocks fetched per batch.                                                                                                                |
| `KEEP_ALIVE_INTERVAL` | `5`        | Seconds between tip checks once caught up.                                                                                               |
| `START_BLOCK`         | `0`        | First block to index **when the database is empty**. Once `block` has rows the indexer resumes from the highest one and this is ignored. |

The API reads `DATABASE_URL` too, plus its own `API_HOST` / `API_PORT` /
`API_MAX_CONNECTIONS` / `STATS_REFRESH_INTERVAL` / `STATISTICS_REFRESH_INTERVAL`; see the
[api README](api/README.md).

Networks:

| Network      | chainId  | Example RPC                        |
| ------------ | -------- | ---------------------------------- |
| Mainnet      | 47763    | https://mainnet-1.rpc.banelabs.org |
| Testnet (T4) | 12227332 | https://neoxt4seed1.ngd.network    |

## Run

```bash
cargo run --release --package neo-scope-indexer
cargo run --release --package neo-scope-api
```

Indexer startup waits for the node to answer `web3_clientVersion` (12 attempts, 5s apart),
validates `CHAIN_ID` if set, then applies migrations. It performs an initial sync up to the
current node height and keeps following the chain from there.

The API binds `0.0.0.0:8083` and serves whatever is already indexed; it applies no migrations,
so run the indexer at least once first. Check it with:

```bash
curl http://localhost:8083/v1/neox/stats
```

### Migrations

There is a single migration, [V1\_\_create_initial_tables.sql](lib/db/migrations/V1__create_initial_tables.sql),
applied automatically from `lib/db/migrations/` on startup. Applied versions are tracked in
`_sqlx_migrations`, so **changing the migration does not re-apply it**. To pick up a schema
change you have to recreate the database:

```sql
DROP DATABASE neox; CREATE DATABASE neox;
```

## Layout

Cargo workspace with three crates:

| Crate               | Path                 | Contents                                                              |
| ------------------- | -------------------- | --------------------------------------------------------------------- |
| `neo-scope-db`      | [lib/](lib/)         | Database layer: row models, migrations, MySQL access.                 |
| `neo-scope-indexer` | [indexer/](indexer/) | The indexing binary: RPC client, conversion engine, config, startup.  |
| `neo-scope-api`     | [api/](api/)         | The HTTP binary: read-only `/v1/neox` endpoints over the same schema. |

The `neo-scope-db` crate has no `src/` directory. Its root is [lib/lib.rs](lib/lib.rs), pointed at
explicitly from its `Cargo.toml`.

The two binaries are independent processes sharing only the database. The indexer writes and
owns the migrations; the API only reads.

## Schema conventions

Tables are singular and every SQL identifier is camelCase. Each table has a surrogate
`AUTO_INCREMENT` primary key named `id<Table>Pk` (`idBlockPk`, `idTransactionPk`,
`idAddressPk`, …). The chain-level natural key of each table is kept as a `UNIQUE`
constraint, so re-indexing stays idempotent and foreign keys still reference the natural
columns.

Rust struct fields stay snake_case, which is the language convention. Nothing maps by name;
the sqlx binds are positional.

> On Windows, MySQL defaults to `lower_case_table_names=1`, which stores _table_ names
> folded to lowercase: `tokenTransfer` shows up as `tokentransfer` in `SHOW TABLES`. Queries
> are unaffected (name matching is case-insensitive) and **column** names keep their
> camelCase.

## How syncing behaves

- **A batch is all-or-nothing.** Blocks, transactions, logs, contracts, addresses, tokens and
  token transfers are written in one SQL transaction, so a range is either fully indexed or
  not indexed at all.
- **Holes stop the batch, they don't become gaps.** Only the contiguous prefix of a range
  whose blocks _and_ receipts both arrived is committed. A block whose receipts failed would
  be stored without its logs, so the batch is cut short of it and the next pass retries it.
- **Re-indexing is safe.** Rows go in with `INSERT IGNORE`, and `address` / `token` upsert, so
  re-runs and backfills only ever widen what is known: `firstSeenBlock` shrinks toward the
  earliest sighting, `isContract` never downgrades, and NULL metadata gets filled in.
- **Token metadata is fetched at most once per token.** The set of already-described tokens is
  loaded from the `token` table at startup, so restarts do not re-issue `eth_call`s.
- **RPC concurrency is capped** at 10 in-flight requests, so a wide batch does not overwhelm
  the node.
- **Lock contention is retried.** MySQL errors 1205 (lock wait timeout) and 1213 (deadlock)
  retry the whole transaction, up to 6 attempts with 50ms→800ms backoff.

## Test

```bash
cargo test --workspace
cargo clippy --workspace --all-targets
```

Tests live in files separate from the code they exercise, under
[indexer/src/engine/tests/](indexer/src/engine/tests/) and
[api/src/domains/tests/](api/src/domains/tests/), wired in with `#[path]`. See
[CLAUDE.md](CLAUDE.md) for why that shape and not a top-level `tests/` directory.

The fixtures in `tests/converters.rs` are real `eth_getBlockReceipts` payloads from Neo X T4
(blocks 306985 and 307936), kept verbatim so the serde models stay honest about the node's
actual response shape.

## Contributing

Contributions are welcome. [CONTRIBUTING.md](CONTRIBUTING.md) covers the development setup,
the checks a pull request must pass, and the coding conventions, including the rule that
code carries **no comments**.

## License

Copyright (c) 2026 Neo News Today.

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion
in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above,
without any additional terms or conditions.
