# Contributing to tpt-data-parsers

Thanks for your interest in improving these parsers! This document covers local
setup, the verification commands CI runs, and the project conventions.

## Prerequisites

- A recent stable Rust toolchain (`rustup toolchain install stable`).
- `rustfmt` and `clippy` components: `rustup component add rustfmt clippy`.
- The minimum supported Rust version (MSRV) is **1.71**.

## Local verification

CI runs exactly this sequence; mimic it locally before pushing:

```bash
cargo fmt --all -- --check          # formatting must be clean
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --all-features
cargo test --no-default-features -p tpt-mime-pure   # tpt-mime-pure is no_std
```

Per-crate checks also work, e.g. `cargo test -p tpt-cron-parse`. The
`tpt-jsonl-stream` crate has an optional `simd` feature (`simd-json`) behind
which the fast path is gated; default builds use the pure-Rust path.

## Project conventions

- **Workspace, not per-crate, metadata.** Shared values (`version`, `edition`,
  `rust-version`, `license`, `authors`, `repository`, and the `serde` /
  `serde_json` dependencies) live in the root `Cargo.toml` under
  `[workspace.package]` / `[workspace.dependencies]` and are referenced via
  `workspace = true`. Do not hardcode them in member crates.
- **Zero / low allocation.** Parsers should avoid allocating in the hot path.
  `tpt-cron-parse`, `tpt-mime-pure`, and `tpt-logfmt-parse` intentionally have
  **no runtime dependencies** — keep it that way. `tpt-jsonl-stream` only depends
  on `serde_json`.
- **`no_std` aware.** `tpt-mime-pure` must stay `no_std` compatible (its `std`
  feature is default). Do not introduce `std`-only APIs without a feature gate.
- **Precise errors.** Parsers report byte offsets, line numbers, or structure
  paths rather than opaque failures.
- **Docs.** Each crate has its own `README.md` and is documented on crates.io;
  update both when the public API or examples change. Crates use
  `#![warn(missing_docs)]`.
- **Changelog.** Keep `CHANGELOG.md` current (Keep a Changelog format) and bump
  the shared `version.workspace` before publishing.

## Adding a new crate

1. Add it under `members` in the root `Cargo.toml` and derive shared metadata via
   `workspace = true`.
2. Add a row to the crate table in the root `README.md`.
3. Add a `## Per-crate commands` / verification note to `AGENTS.md` if it has
   special build flags.
4. Wire publishing into `.github/workflows/publish.yml` if it is a library.

## Publishing

Publishing is automated: each crate is released by the `publish.yml` workflow when
a `v*.*.*` git tag is pushed. You do not publish from your machine — just bump the
version, update the changelog, open a PR, and tag once CI is green.
