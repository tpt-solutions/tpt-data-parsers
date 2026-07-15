# AGENTS.md

Rust Cargo workspace: 5 independent parser crates under `tpt-*/`, resolver = "2",
edition 2021, MSRV **1.70** (set in `[workspace.package]`). Shared metadata and
deps come from the root `Cargo.toml` via `workspace = true`; do not hardcode them
in member crates.

## Verify before committing

CI runs this exact sequence (mimic locally):

1. `cargo fmt --all -- --check` — formatting must be clean.
2. `cargo clippy --all-targets --all-features -- -D warnings` — warnings are **denied**, must be zero.
3. `cargo test --all --all-features` — full test run.
4. `cargo test --no-default-features -p tpt-mime-pure` — `tpt-mime-pure` is **no_std**;
   its `std` feature is default, so explicitly test the no-default build.

## Per-crate commands

- Build/test one crate: `cargo test -p <crate>`, e.g. `cargo test -p tpt-cron-parse`.
- `tpt-jsonl-stream` has an optional `simd` feature (`simd-json`); default build is
  the pure-Rust, zero-allocation path. Gate simd changes behind the feature.

## Publishing

No workspace-wide release. Each crate is published separately by the `publish.yml`
workflow when a `v*.*.*` git tag is pushed:

```
cargo publish -p tpt-logfmt-parse
cargo publish -p tpt-cron-parse
cargo publish -p tpt-mime-pure
cargo publish -p tpt-jsonl-stream
cargo publish -p tpt-geo-geojson
```

Bump each crate's version (in root `Cargo.toml` `version.workspace`, shared) and tag
before pushing. crates.io tokens are repo secrets; publishing from CI only.

## Conventions

- Crates aim for zero/low-allocation, pure-Rust parsing; keep deps minimal
  (`tpt-cron-parse`, `tpt-mime-pure`, `tpt-logfmt-parse` have no runtime deps).
- Each crate has its own `README.md` and is documented on crates.io; update both
  when the public API or examples change.
