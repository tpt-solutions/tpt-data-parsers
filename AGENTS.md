# AGENTS.md

Rust Cargo workspace (resolver "2", edition 2021, MSRV **1.71**). Six members under
`tpt-*/`: five published parser **libraries** plus `tpt-cli` (a binary, not a library).

Crates are independent — they do **not** depend on each other. Each library is a single
`src/lib.rs` plus a `tests/` file and its own `README.md`; `tpt-cli` (`src/main.rs`) is
the only binary and just wraps the five libraries via `clap`.

## Verify before committing

CI runs exactly this (mimic locally, in this order):

1. `cargo fmt --all -- --check`
2. `cargo clippy --all-targets --all-features -- -D warnings` — warnings denied, must be zero
3. `cargo test --all --all-features`
4. `cargo test --no-default-features -p tpt-mime-pure` — `tpt-mime-pure` is `no_std`; `std` is the default feature, so the no-default build must be tested explicitly

`--all-targets` also compiles each crate's `benches/bench.rs` (criterion, `harness = false`),
so bench code must be warning-free too. One crate: `cargo test -p <crate>`; one test:
`cargo test -p <crate> <test_name>`.

## Conventions that differ from defaults

- Shared metadata (`version`, `edition`, `rust-version`, `license`, `repository`) and the
  `serde`/`serde_json` deps live in root `Cargo.toml` and are referenced with `workspace = true`.
  **Never hardcode them in a crate's `Cargo.toml`.** There is one shared `version.workspace` —
  bump it once in the root before a release.
- **Zero runtime dependencies by design** for `tpt-cron-parse`, `tpt-mime-pure`,
  `tpt-logfmt-parse`. Do not add a dependency to these without discussion.
- Each crate has `#![warn(missing_docs)]` and
  `#![doc = include_str!("../README.md")]`. The crate `README.md` is both the doctest
  source and the crates.io landing page — update both when the public API changes.
- Optional features: `tpt-cron-parse` `chrono` (enables `next_after`/`upcoming`),
  `tpt-jsonl-stream` `simd`. `cron`/`jsonl` set `[package.metadata.docs.rs] all-features = true`
  so their docs build with features enabled.

## Gotchas

- `tpt-cli` is **not published**: it has `publish = false` and is deliberately absent from
  `publish.yml`. Do not add it there or treat it as a library.
- `fuzz/` dirs under each crate are excluded from the workspace and require nightly +
  `cargo-fuzz`. Do not run ordinary `cargo` commands inside them.
- `.kilo/` is gitignored local Kilo state — do not force-add it.

## Publishing

Per-crate, triggered by pushing a `v*.*.*` git tag (`publish.yml`). Bump the shared
`version` in root `Cargo.toml`, then tag. Publishing uses repo-secret crates.io tokens
from CI only — never locally. `CLAUDE.md` has the full command reference.
