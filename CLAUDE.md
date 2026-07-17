# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Rust Cargo workspace of 5 independent, published-to-crates.io parser crates,
each a single-purpose, low-dependency parser (`tpt-jsonl-stream`,
`tpt-geo-geojson`, `tpt-logfmt-parse`, `tpt-cron-parse`, `tpt-mime-pure`).
Every crate lives entirely in one `src/lib.rs` plus a `tests/` integration
test file and its own `README.md`. There is no shared internal library —
crates do not depend on each other.

## Commands

CI runs this exact sequence; mimic it locally before considering work done:

```
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --all-features
cargo test --no-default-features -p tpt-mime-pure
```

- Single crate: `cargo test -p <crate>` (e.g. `cargo test -p tpt-cron-parse`).
- Single test: `cargo test -p <crate> <test_name>`.
- `tpt-mime-pure` is `no_std`; its `std` feature is default, so the
  no-default-features build must be tested explicitly (that's what step 4
  above does).
- `tpt-jsonl-stream` has an optional `simd` feature (`simd-json`, via
  `cargo test -p tpt-jsonl-stream --features simd`); default build is the
  pure-Rust, zero-allocation path. Gate simd-only changes behind the feature
  and keep the non-simd path working without it.

## Workspace conventions

- Shared metadata (`version`, `edition`, `license`, `repository`, `rust-version`)
  and shared deps (`serde`, `serde_json`) live in root `Cargo.toml` under
  `[workspace.package]` / `[workspace.dependencies]`. Member crates reference
  them with `workspace = true` — never hardcode these values in a crate's own
  `Cargo.toml`.
- MSRV is **1.71**; avoid syntax/stdlib features newer than that.
- `tpt-cron-parse`, `tpt-mime-pure`, and `tpt-logfmt-parse` have **zero
  runtime dependencies** by design — don't add one without discussing it.
- Each crate targets zero/low-allocation parsing in the hot path, especially
  `tpt-logfmt-parse` (zero-copy) and `tpt-jsonl-stream`.
- Parsers report precise errors — byte positions, line numbers, or structure
  paths — rather than opaque failures. Follow this pattern (see each crate's
  `*Error` type, e.g. `CronError` with position/field/expected/found,
  `JsonlError` with line number, `GeoError` with a path string) when adding
  new error variants.
- Every crate has `#![warn(missing_docs)]` and
  `#![doc = include_str!("../README.md")]` at the top of `lib.rs` — new public
  items need doc comments, and the crate's `README.md` doubles as its
  crates.io landing page and doctest source. Update both the code and the
  README when the public API changes.
- New features/fixtures follow the existing per-crate pattern: unit tests
  inline in `lib.rs`, an integration test in `tests/`, and (for
  `tpt-jsonl-stream` / `tpt-geo-geojson`) fixture files under `tests/fixtures/`.

## Publishing

No workspace-wide release; each crate is published independently by the
`publish.yml` GitHub Actions workflow when a `v*.*.*` tag is pushed. Bump the
shared `version` in root `Cargo.toml` and tag before pushing — publishing
itself only happens from CI using repo-secret crates.io tokens, not locally.
