# Neo X API

Read-only HTTP API over the data written by the [indexer](../indexer/). Every response is
served straight from MySQL; the API never talks to a Neo X node.

Everything lives under `/v1/neox`. Every domain follows the same route layout, pagination
shape and error body, described below.

## Run

```bash
cargo run --release --package neo-scope-api
```

Requires the same `DATABASE_URL` the indexer uses. The API does **not** run migrations:
start the indexer at least once first, or the tables will not exist.

| Variable                 | Default   | Meaning                                                    |
| ------------------------ | --------- | ---------------------------------------------------------- |
| `DATABASE_URL`           | *required*| MySQL DSN, e.g. `mysql://user:pass@localhost:3306/neox`.    |
| `API_HOST`               | `0.0.0.0` | Bind address.                                               |
| `API_PORT`               | `8083`    | Bind port. The N3 API uses 8082, so both can run together.  |
| `API_MAX_CONNECTIONS`    | `10`      | MySQL pool size.                                            |
| `STATS_REFRESH_INTERVAL` | `300`     | Seconds between `/stats` count refreshes (see **Stats**).   |
| `STATISTICS_REFRESH_INTERVAL` | `900` | Seconds between `/stat/*` refreshes (see **Statistics**).   |

CORS allows any origin for `GET` only.

## Endpoints

| Method | Path                                  | Returns                                            |
| ------ | ------------------------------------- | -------------------------------------------------- |
| GET    | `/v1/neox/block`                      | Blocks, newest first.                               |
| GET    | `/v1/neox/block/{number\|hash}`       | One block, by height or by hash.                    |
| GET    | `/v1/neox/block/{number\|hash}/transactions` | That block's transactions.                   |
| GET    | `/v1/neox/transaction`                | Transactions, newest first.                         |
| GET    | `/v1/neox/transaction/{hash}`         | One transaction, receipt fields included.           |
| GET    | `/v1/neox/address`                    | Known addresses, most recently discovered first.    |
| GET    | `/v1/neox/address/{address}`          | One address, with its contract/token record if any. |
| GET    | `/v1/neox/address/{address}/transactions` | Transactions where it is sender or recipient.   |
| GET    | `/v1/neox/address/{address}/transfers`| Token transfers where it is sender or recipient.    |
| GET    | `/v1/neox/contract`                   | Deployed contracts, newest first.                   |
| GET    | `/v1/neox/contract/{address}`         | One contract, with its creation data.               |
| GET    | `/v1/neox/contract/{address}/invocations` | Daily invocation counts (see **Invocations**).  |
| GET    | `/v1/neox/token`                      | Token contracts, newest first.                      |
| GET    | `/v1/neox/token/{address}`            | One token, with its metadata.                       |
| GET    | `/v1/neox/token/{address}/transfers`  | That token's transfers.                             |
| GET    | `/v1/neox/log`                        | Event logs, newest first.                           |
| GET    | `/v1/neox/stats`                      | Current height and table counts.                    |
| GET    | `/v1/neox/stat/neox-statistics`       | Per-period counts (see **Statistics**).             |
| GET    | `/v1/neox/stat/network-statistics`    | Totals plus a rolling week (see **Statistics**).    |

The list endpoints on `transaction`, `address`, `contract` and `token` exist because an
explorer front end needs to browse every domain. They cost nothing extra: each is the same
paginated query the detail endpoints build on.

`{number|hash}` is one path segment read two ways: as a height when it parses as an integer,
as a block hash otherwise. There is no separate `/block/hash/…` route. These are the same
block:

```
GET /v1/neox/block/9372613
GET /v1/neox/block/0x1f1dd583640779ec12f174a61ec04064f7865668a0cce63c1cee4bf24a202898
GET /v1/neox/block/1f1dd583640779ec12f174a61ec04064f7865668a0cce63c1cee4bf24a202898
```

The same holds for `/block/{number|hash}/transactions`. Anything that is neither a height nor
a 32-byte hex hash is `400`; a well-formed one that was never indexed is `404`.

### Query parameters

| Parameter | Applies to      | Meaning                                                      |
| --------- | --------------- | ------------------------------------------------------------ |
| `id`      | every list      | Pagination cursor. Pass back what `next_page_params.id` gave. |
| `limit`   | every list      | Page size, `1` to `100`, default `10`. Out-of-range values clamp.|
| `type`    | `/token`        | `ERC20`, `ERC721` or `ERC1155` (lowercase accepted).          |
| `address` | `/log`          | Emitter. Same casing rules as a path address.                 |
| `topic0`  | `/log`          | Event signature hash, 32 bytes.                               |
| `from_block` / `to_block` | `/log` | Inclusive block range.                                     |
| `started_at` / `ended_at` | `/log`, `/contract/{address}/invocations` | Inclusive time window. |

`started_at` and `ended_at` accept three forms, and `ended_at` reads a bare date as the **end**
of that UTC day:

| Form           | Example                | Read as                                        |
| -------------- | ---------------------- | ---------------------------------------------- |
| `YYYY-MM-DD`   | `2026-08-11`           | `00:00:00Z`, or `23:59:59Z` for `ended_at`.     |
| RFC 3339       | `2026-08-11T14:00:00Z` | Exactly that instant. Offsets are honoured.     |
| Epoch integer  | `1786406400`           | Seconds. Values ≥ 10¹¹ are read as milliseconds, so a consumer that already speaks the millisecond-based N3 API does not silently ask for the year 58000. |

Both are optional and either may stand alone. `started_at` after `ended_at` is `400`. A window
that no block falls into is an empty result, not an error. Internally each bound is resolved to
a block number once, so the filter runs on an indexed integer column rather than a per-row join.

## Pagination

Lists are keyset-paginated and return newest first:

```json
{
  "items": [ /* … */ ],
  "next_page_params": { "id": 320674 }
}
```

`next_page_params` is `null` on the last page. To get the next page, send its `id` back as
`?id=`. The cursor is the position of the **last item on the page**, so paging never skips or
repeats a row, as verified in [tests/model.rs](src/domains/tests/model.rs).

Treat `id` as opaque. It is the block height for `/block` and the table's primary key
everywhere else, which is why it does not appear in the item bodies.

Ordering is by insertion for everything except `/block`, which orders by height. For a
forward-running sync the two are the same; after a backfill of an older range, that range's
rows sort as if they were newly seen.

## Types

- **Wei and other u256 values are decimal strings:** `value`, `gas_price`, `base_fee_per_gas`,
  `max_fee_per_gas`, `max_priority_fee_per_gas`, `effective_gas_price`, a transfer's `amount`
  and its `token_id`. They do not fit in a JSON number and are never truncated to one.
- Values that do fit (heights, gas limits, `timestamp`, `nonce`, `tx_index`, `log_index`)
  are JSON numbers.
- `timestamp` is the block's, in **seconds** (the N3 API uses milliseconds; this one reports
  what the EVM stores).
- Addresses and hashes come back lowercase and `0x`-prefixed. Requests may use any casing,
  with or without the prefix, so EIP-55 checksummed input works. This applies everywhere a hash
  or address is a path segment: `/block/{hash}`, `/transaction/{hash}`, `/address/{address}`,
  `/contract/{address}`, `/token/{address}`.
- A field the indexer could not fill is `null`, never omitted: a pending-style transaction has
  no `status`, a contract creation has no `to_address`, a token whose getter reverted has no
  `symbol`.

## Errors

Every error is `{"error": "…"}` with a matching status:

| Status | When                                                              |
| ------ | ----------------------------------------------------------------- |
| `400`  | Malformed address, hash, cursor, `limit` or `type`.                |
| `404`  | The row does not exist, or the route does not.                     |
| `500`  | The database failed. The detail is logged, not returned.           |

A list whose subject exists but has no rows is `200` with `"items": []`. An empty page is not
an error. Asking for the transactions of a block that does not exist is `404`.

`/contract/{address}/invocations` is the one exception to the `404`-on-unknown-subject rule;
see below.

## Invocations

```
GET /v1/neox/contract/0xebe9e10368ee15c8d94cda74a39e5312b0d9bd18/invocations
```

```json
{ "2026-08-10": 412, "2026-08-11": 87 }
```

Keys are UTC days, `YYYY-MM-DD`, ascending. Days with no activity are absent rather than zero.
The N3 API's equivalent returns `YYYYMMDD`; this one returns the ISO form, which `Date.parse`
reads directly.

**A transaction counts as one invocation of a contract when the contract is either the
transaction's direct recipient or the emitter of one of its logs.** Transactions are counted
once per contract per day, so a transaction that emits five events from the same contract is
one invocation, and one that touches three contracts is one invocation of each.

Counting only `toAddress` would miss everything reached through a router or proxy. On the T4
data this is not a corner case: contract `0xebe9e103…` is the direct recipient of exactly **1**
transaction and the emitter in **43177**. Counting it by recipient alone would under-report it
by four orders of magnitude. Both halves are needed in the other direction too: contracts that
are called directly but emit no event only show up on the `toAddress` side.

What it still misses is a call that neither targets the contract directly nor emits an event.
Catching those needs `debug_traceTransaction` and an internal-transaction table, which this
indexer does not have. See the [root README](../README.md).

The address only has to be a well-formed address. A contract deployed from inside another
contract is not in the `contract` table (the indexer cannot see those without tracing) but its
logs are indexed, so it still answers. An address with nothing to count is `{}`, not `404`.

## Stats

```json
{
  "latest_block": 9372613,
  "total_blocks": 9372614,
  "total_transactions": 320682,
  "total_logs": 300759,
  "total_addresses": 19557,
  "total_contracts": 5231,
  "total_tokens": 1031,
  "total_token_transfers": 42124,
  "counts_updated_at_ms": 1785948374132
}
```

`latest_block` is read live on every request, since it is a single index lookup. The counts are
`COUNT(*)` over whole tables, which takes seconds on a full chain, so they come from a cache
refreshed every `STATS_REFRESH_INTERVAL` seconds and stamped with `counts_updated_at_ms`.
The refresh skips the scan entirely when the height has not moved since the last one. Until
the first refresh finishes the counts are zero.

## Statistics

Two aggregate endpoints, both mirroring their N3 counterparts field for field so a front end
that already reads `/v1/stat/n3-statistics` and `/v1/stat/network-statistics` needs only a new
URL.

```
GET /v1/neox/stat/neox-statistics
```

```json
{
  "updated_at_ms": 1786473164950,
  "data": {
    "contracts":   {"total":5246,"today":0,"yesterday":1,"this_week":1,"last_week":15,"this_month":16,"last_month":71},
    "addresses":   {"total":19591,"today":3,"yesterday":4,"this_week":7,"last_week":33,"this_month":40,"last_month":140},
    "invocations": {"total":343416,"today":193,"yesterday":312,"this_week":505,"last_week":1586,"this_month":2555,"last_month":12392}
  }
}
```

```
GET /v1/neox/stat/network-statistics
```

```json
{
  "total_transactions": 322040,
  "total_addresses": 19591,
  "total_contracts": 5246,
  "current_week_transactions": 1290,
  "current_week_addresses": 34,
  "current_week_contracts": 15
}
```

What each row counts:

| Row           | A unit is                                                                    |
| ------------- | ---------------------------------------------------------------------------- |
| `contracts`   | A contract creation, placed at its creation block.                            |
| `addresses`   | An address seen for the first time, placed at its `firstSeenBlock`. `total` is every address ever seen. |
| `invocations` | A (contract, transaction) pair, by the same rule as **Invocations** above.    |

Periods are UTC. `today`, `this_week` and `this_month` run from their boundary up to *now*;
`yesterday`, `last_week` and `last_month` are whole closed periods. Weeks start Monday. The
rolling `current_week_*` of `network-statistics` is a different window on purpose: the last
seven days ending now, matching what the N3 endpoint reports.

Each period boundary is resolved to a block number once per refresh, so the counting queries
compare indexed integers instead of joining every row to `block`.

Both come from a cache refreshed every `STATISTICS_REFRESH_INTERVAL` seconds, and are zero
until the first refresh finishes. Unlike `/stats`, the refresh is **not** skipped when the
height has not moved: the periods are relative to the wall clock, so yesterday's numbers have
to move at midnight on an idle chain too.

The `invocations` row is the expensive one. It deduplicates a union of the whole `transaction`
and `log` tables, measured at ~8s over 322k transactions and 302k logs, and it grows with them.
That is why it has its own interval instead of riding on `STATS_REFRESH_INTERVAL`; raise it
further on a large chain.

## Layout

Each domain is a folder under [src/domains/](src/domains/) with the same four files:

| File            | Holds                                                        |
| --------------- | ------------------------------------------------------------ |
| `controller.rs` | Routes and extractors. Turns a request into a service call.   |
| `service.rs`    | Input parsing, existence checks, cross-domain composition.    |
| `query.rs`      | The SQL.                                                      |
| `model.rs`      | The response types.                                           |

Response structs derive both `Serialize` and `sqlx::FromRow`, and the SQL aliases each
camelCase column to the snake_case field name. There is no separate row type to convert
through, because nothing needs converting: what the schema stores is what the API returns.

Queries that match an address against *either* side of a row (an address's transactions and
its transfers) do not use `WHERE from = ? OR to = ?`. MySQL answers that with an index merge
followed by a filesort over every match, which on an address with ~19k transactions measured
at 1.6s per page. They instead union two single-index lookups, each already ordered by primary
key, and join the resulting page back for the full rows: same result, 1.6s → ~1ms. The shared
builder is in [src/domains/sql.rs](src/domains/sql.rs).

## Test

```bash
cargo test --package neo-scope-api
```

Tests cover the pure logic: cursor arithmetic and address/hash parsing. Following
[CLAUDE.md](../CLAUDE.md), they live beside the module in `tests/` and are wired in with
`#[path]`, since a binary crate cannot be imported by an integration test.

## Not covered

No response embeds a transaction's logs; `/log?from_block=…` is the way to reach them.
Balances, internal transactions and pending transactions are not indexed at all, so nothing
serves them; the consequence for invocation counting is spelled out under **Invocations**. See
the [root README](../README.md).
