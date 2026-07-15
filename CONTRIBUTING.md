# Contributing to tpt-data-parsers

Thanks for your interest in contributing! This is a Cargo workspace of small,
independent parser crates. Each crate is published separately to
[crates.io](https://crates.io).

## Getting started

```sh
git clone https://github.com/tpt-solutions/tpt-data-parsers
cd tpt-data-parsers
cargo test --workspace --all-features
cargo test --no-default-features -p tpt-mime-pure   # no_std build
```

## Before opening a PR

CI runs this exact sequence — please run it locally first:

1. `cargo fmt --all -- --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test --all --all-features`
4. `cargo test --no-default-features -p tpt-mime-pure`

A `Hygiene` workflow additionally enforces that the `authors` field resolves to
**TPT Solutions** and that no forbidden email domains appear in tracked files.

## Conventions

- **No new runtime dependencies** for `tpt-cron-parse`, `tpt-mime-pure`, and
  `tpt-logfmt-parse` — they are intentionally dependency-free.
- **MSRV is 1.70** (`rust-version` in the root `Cargo.toml`). Avoid stdlib/syntax
  features newer than that.
- Shared metadata and dependencies live in the root `Cargo.toml` via
  `workspace = true`; never hardcode them in a member crate.
- Every public item needs a doc comment (`#![warn(missing_docs)]` is on). The
  crate `README.md` doubles as the crates.io landing page and doctest source —
  update both when the public API changes.
- Prefer zero/low allocation in the hot path; parsers report precise errors
  (byte positions, line numbers, or structure paths).

## Adding a feature

- Add unit tests inline in `src/lib.rs` and an integration test under `tests/`.
- For `tpt-jsonl-stream` / `tpt-geo-geojson`, add fixture files under
  `tests/fixtures/`.
- Bump the shared `version` in the root `Cargo.toml` and tag `v*.*.*` to publish.

## License

By contributing you agree that your contributions are licensed under the same
terms as the project (Apache-2.0 OR MIT).
