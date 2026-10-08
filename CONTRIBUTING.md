# Contributing to neo-scope

Thanks for your interest in neo-scope. Bug reports, fixes, features and documentation
improvements are all welcome.

By participating you agree to follow the [Code of Conduct](CODE_OF_CONDUCT.md). To report a
security vulnerability, **do not open a public issue**; follow [SECURITY.md](SECURITY.md).

## Before you start

- **Bugs:** open an issue with the steps to reproduce, what you expected and what happened.
  Include the neo-scope version or commit, the network (mainnet / testnet T4) and relevant
  log lines.
- **Features and larger changes:** open an issue first and describe the idea. Agreeing on the
  approach before writing code saves everyone a rewrite.
- **Small fixes** (typos, docs, obvious bugs) can go straight to a pull request.

Issues labelled [`good first issue`](https://github.com/neonewstoday/neo-scope/labels/good%20first%20issue)
are a good place to start.

## Development setup

You need:

- [rustup](https://rustup.rs). The repository pins the toolchain in `rust-toolchain.toml`
  (currently 1.97.1, with `rustfmt` and `clippy`), and rustup installs it on the first
  `cargo` command. The minimum supported version is 1.94.
- MySQL 8 with InnoDB.
- A Neo X JSON-RPC endpoint. The public testnet T4 node in `.env.example` works.

```bash
git clone https://github.com/neonewstoday/neo-scope.git
cd neo-scope
cp .env.example .env
mysql -u root -p -e "CREATE DATABASE neox_testnet"
cargo build --workspace
```

Run the indexer, then the API in a second terminal:

```bash
cargo run --package neo-scope-indexer
cargo run --package neo-scope-api
```

The indexer applies migrations on startup; the API never does, so start the indexer first.
Configuration is documented in the [README](README.md#configure) and the
[api README](api/README.md).

> **Windows:** build with the MSVC toolchain from PowerShell or `cmd`. Inside Git Bash,
> coreutils' `link` shadows the MSVC `link.exe` and every build script fails with
> `link: extra operand`.

## Before opening a pull request

All of these must pass. CI runs the same commands:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI also checks that the workspace builds on the minimum supported Rust version
(`cargo +1.94 check --workspace --all-targets`), that the Docker image builds, and runs
[cargo-deny](https://github.com/EmbarkStudios/cargo-deny) against [deny.toml](deny.toml):
known vulnerabilities, yanked crates, and dependency licenses outside the allow list fail the
build. If you add a dependency, run `cargo deny check` locally
(`cargo install cargo-deny --locked`).

Add or update tests for any behavior you change.

## Code conventions

### No comments

**Code in this repository carries no comments**, doc comments (`///`, `//!`) included. Names
explain the code; a comment that restates it goes stale after the next refactor and nobody
tests it.

A comment is allowed only when it records something the code cannot express:

1. A non-obvious constraint: a version pin, an ordering that looks arbitrary, a workaround
   for a third-party bug.
2. A directive a tool requires, which only works as a comment, such as `// SAFETY:` on an `unsafe`
   block. Attributes such as `#[allow(...)]` are not comments and are used normally.
3. An external reference that defines a format: an EIP, RFC or spec section.
4. A legal or license header.

Instead of commenting, extract a function with a descriptive name, introduce a named
constant, rename, or write a test named after the edge case. Explain *why* a change was made
in the commit message and PR description, not in the code.

### Database

- Tables are **singular** and every SQL identifier is **camelCase**.
- Every table has an `AUTO_INCREMENT` primary key named `id<Table>Pk` (`idBlockPk`,
  `idTokenTransferPk`). The table's natural key is a `UNIQUE` constraint, which keeps
  re-indexing idempotent and lets foreign keys keep pointing at natural columns.
- Rust struct fields stay snake_case. sqlx binds are positional, so nothing maps by name.

### Migrations

Migrations live in [lib/db/migrations/](lib/db/migrations/) and are compiled into the
binary. The `MIGRATIONS` list in [lib/db/migrate.rs](lib/db/migrate.rs) embeds each file
with `include_str!`, so a new migration must also be added there, in version order. The
indexer applies them on startup. Applied versions are recorded in `_sqlx_migrations`, so
editing an applied migration has no effect on an existing database.

- **Before 0.1.0:** there is a single migration, `V1__create_initial_tables.sql`, and schema
  changes edit it directly. Recreate your database to pick them up.
- **From 0.1.0 on:** `V1` is frozen. Every schema change is a new file,
  `V<n>__<description>.sql`, with the next version number. Never edit a migration that has
  shipped in a release.

### API

- Each domain in [api/src/domains/](api/src/domains/) has exactly four files:
  `controller.rs` (routes), `service.rs` (input validation and composition), `query.rs`
  (SQL) and `model.rs` (response types).
- Routes, pagination (`next_page_params`) and the error body (`{"error": …}`) are the same
  across domains. Follow the existing shape rather than inventing a new one; the contract is
  documented in [api/README.md](api/README.md).
- Response structs derive both `Serialize` and `sqlx::FromRow`. The SQL aliases each
  camelCase column to the snake_case field name, so there is no intermediate row type.
- u256 values (wei, `amount`, `tokenId`) are decimal strings, in storage and in responses.
- SQL built with `format!` must go through `sqlx::AssertSqlSafe`. Only interpolate constants
  from the code: table and column names, clauses with `?` placeholders. **Every value that
  comes from a request goes through `.bind()`.**

### Tests

Tests live in their own files, never in the file under test. `neo-scope-indexer` and
`neo-scope-api` are binary crates (integration tests cannot import from them), and some of
what is tested is private, so each module declares its test file with `#[path]`:

```rust
#[cfg(test)]
#[path = "tests/abi.rs"]
mod tests;
```

The files sit in a `tests/` directory next to the module (for example
[indexer/src/engine/tests/](indexer/src/engine/tests/)) and remain submodules of the module
under test, so `use super::*` and private items work.

## Commits

Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<optional scope>): <summary in the imperative, lowercase, no period>

<optional body: what changed and why>
```

Types: `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`.
Scopes, when useful: `indexer`, `api`, `db`.

```
feat(api): add holder count to token detail
fix(indexer): retry receipts when the node returns null
docs: document the migration policy
```

Mark breaking changes (schema changes that require re-indexing, or API response changes)
with `!` after the type (`feat(db)!: …`) and a `BREAKING CHANGE:` footer explaining the
migration.

## Pull requests

1. Fork the repository and create a branch from `main`.
2. Keep each PR focused on one change. Unrelated cleanups go in a separate PR.
3. Make sure the checks in [Before opening a pull request](#before-opening-a-pull-request)
   pass.
4. Fill in the PR template: what changed, why, and how you tested it. Link the issue it
   closes.
5. A maintainer reviews it. Address the feedback by pushing new commits; PRs are squash-merged,
   so the PR title becomes the commit message and must follow Conventional Commits.

## License

neo-scope is dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE). Unless
you explicitly state otherwise, any contribution you intentionally submit for inclusion in
the project, as defined in the Apache-2.0 license, is dual-licensed as above, without any
additional terms or conditions.
