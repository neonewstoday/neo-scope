# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). While the
version is `0.x`, a minor bump (`0.1` → `0.2`) may include breaking changes to the database
schema or the API.

## [Unreleased]

First public release, planned as `0.1.0`.

### Added

- `neo-scope-indexer`: syncs Neo X blocks, transactions and receipts from a JSON-RPC node into
  MySQL, with initial sync to the chain tip and continuous follow-up.
- Indexing of blocks, transactions (with receipt fields), event logs, top-level contract
  creations, addresses, ERC-20 / ERC-721 / ERC-1155 tokens with on-chain metadata, and token
  transfers, including ERC-1155 batch transfers.
- All-or-nothing batch writes, idempotent re-indexing, `CHAIN_ID` validation on startup, and
  retry of MySQL lock timeouts and deadlocks.
- `neo-scope-api`: read-only HTTP API under `/v1/neox` for blocks, transactions, addresses,
  contracts, tokens, token transfers, logs, counters and per-period statistics, with cursor
  pagination through `next_page_params`.
- Dual license: MIT OR Apache-2.0.
- Contributor documentation: `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, and issue
  and pull request templates.

[Unreleased]: https://github.com/neonewstoday/neo-scope/commits/main
