# neo-scope

Indexer and read-only HTTP API for the Neo X EVM sidechain. Rust workspace with three crates:

| Crate               | Path       | Role                                                                  |
| ------------------- | ---------- | --------------------------------------------------------------------- |
| `neo-scope-db`      | `lib/`     | Database layer: row models, migrations, MySQL access.                 |
| `neo-scope-indexer` | `indexer/` | The syncing binary: RPC client, conversion engine, config, startup.   |
| `neo-scope-api`     | `api/`     | The HTTP binary: read-only endpoints under `/v1/neox`.                |

## Conventions

All coding conventions live in [CONTRIBUTING.md](CONTRIBUTING.md). Read it before changing
code. The ones most often missed:

- **No comments in code**, doc comments included, outside the listed exceptions. Full rule:
  the [no-comments](.claude/skills/no-comments/SKILL.md) skill (`/no-comments`).
- SQL identifiers are camelCase, tables singular, primary keys `id<Table>Pk`.
- Each API domain has exactly `controller.rs`, `service.rs`, `query.rs` and `model.rs`.
- SQL built with `format!` goes through `AssertSqlSafe`; request values always go through
  `.bind()`.
- Tests live in separate files wired in with `#[path = "tests/<name>.rs"]`, never inline.
- Commits follow Conventional Commits.

API contract details (pagination, types, error codes) are in [api/README.md](api/README.md).

## Commands

```bash
cargo build --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run --release --package neo-scope-indexer
cargo run --release --package neo-scope-api
```

Requires Rust 1.94 or newer (sqlx 0.9). Developed on 1.97.

On Windows, run cargo from PowerShell, not Git Bash: Git Bash's `link` shadows the MSVC
`link.exe` and every build script fails with `link: extra operand`.

There is no `.env` in the repo; integration with a live MySQL or RPC node is not covered by
`cargo test`.
