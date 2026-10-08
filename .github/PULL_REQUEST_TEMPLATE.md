## What

<!-- What this PR changes. -->

## Why

<!-- The problem it solves. Link the issue: "Closes #123". -->

## How it was tested

<!-- Commands run, block ranges indexed, requests made. -->

## Checklist

- [ ] The PR title follows [Conventional Commits](https://www.conventionalcommits.org/) (`feat(api): …`, `fix(indexer): …`)
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] Tests added or updated for the changed behavior
- [ ] Docs updated (README, api/README.md) if behavior or the API contract changed
- [ ] Breaking change (schema requires re-indexing, or API response changed) is marked with `!` and explained
