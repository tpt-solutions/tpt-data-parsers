# tpt-data-parsers

A Rust workspace of ultra-fast, low-allocation, streaming parsers for formats the
broader ecosystem ignores.

[![CI](https://github.com/tpt-solutions/tpt-data-parsers/actions/workflows/ci.yml/badge.svg)](https://github.com/tpt-solutions/tpt-data-parsers/actions/workflows/ci.yml)

Each crate is published independently to [crates.io](https://crates.io) and is
designed to be dropped into a pipeline, a log shipper, or a constrained
(`no_std`) target without pulling in a heavy dependency tree.

## Crates

| Crate | Description | crates.io | docs.rs |
|-------|-------------|-----------|---------|
| [`tpt-jsonl-stream`](./tpt-jsonl-stream) | Streaming, constant-memory JSON Lines parser | [![crates.io](https://img.shields.io/crates/v/tpt-jsonl-stream.svg)](https://crates.io/crates/tpt-jsonl-stream) | [![docs.rs](https://docs.rs/tpt-jsonl-stream/badge.svg)](https://docs.rs/tpt-jsonl-stream) |
| [`tpt-geo-geojson`](./tpt-geo-geojson) | Strict, validating GeoJSON parser with path-located errors | [![crates.io](https://img.shields.io/crates/v/tpt-geo-geojson.svg)](https://crates.io/crates/tpt-geo-geojson) | [![docs.rs](https://docs.rs/tpt-geo-geojson/badge.svg)](https://docs.rs/tpt-geo-geojson) |
| [`tpt-logfmt-parse`](./tpt-logfmt-parse) | Zero-copy logfmt (`key=value`) parser | [![crates.io](https://img.shields.io/crates/v/tpt-logfmt-parse.svg)](https://crates.io/crates/tpt-logfmt-parse) | [![docs.rs](https://docs.rs/tpt-logfmt-parse/badge.svg)](https://docs.rs/tpt-logfmt-parse) |
| [`tpt-cron-parse`](./tpt-cron-parse) | Cron expression parser with human-readable output | [![crates.io](https://img.shields.io/crates/v/tpt-cron-parse.svg)](https://crates.io/crates/tpt-cron-parse) | [![docs.rs](https://docs.rs/tpt-cron-parse/badge.svg)](https://docs.rs/tpt-cron-parse) |
| [`tpt-mime-pure`](./tpt-mime-pure) | Pure Rust MIME type detection via magic bytes (`no_std`) | [![crates.io](https://img.shields.io/crates/v/tpt-mime-pure.svg)](https://crates.io/crates/tpt-mime-pure) | [![docs.rs](https://docs.rs/tpt-mime-pure/badge.svg)](https://docs.rs/tpt-mime-pure) |
| [`tpt-cli`](./tpt-cli) | Command-line front-end for all five parsers (CLI, not a library) | — | — |

## Which crate do I need?

- **I have newline-delimited JSON** (`.jsonl` / NDJSON) and want to iterate records without
  loading everything into memory → **`tpt-jsonl-stream`**.
- **I have GeoJSON** and need strict validation with precise error locations →
  **`tpt-geo-geojson`**.
- **I'm parsing structured logs** of the form `key=value key="quoted value"` →
  **`tpt-logfmt-parse`**.
- **I have a cron schedule string** and want a human-readable description or precise parse
  errors → **`tpt-cron-parse`**.
- **I need to identify a file's type** from its bytes or extension (no `file` binary) →
  **`tpt-mime-pure`**.
- **I want a ready-made command-line tool** to detect, validate, or parse files without
  writing code → **`tpt-cli`**.

If you only have a filename/path rather than raw bytes, `tpt-mime-pure`'s
`detect_by_extension` is the right tool. If you have raw bytes, use `detect`.

## Design principles

- **Zero / low allocation** where it matters — most crates parse without
  allocating in the hot path (`tpt-logfmt-parse`, `tpt-jsonl-stream`).
- **Minimal dependencies** — `tpt-cron-parse`, `tpt-mime-pure`, and
  `tpt-logfmt-parse` have no runtime dependencies; `tpt-jsonl-stream` only
  depends on `serde_json` (with an optional `simd` fast path).
- **Precise errors** — parsers report byte positions, line numbers, or
  structure paths instead of opaque failures.
- **`no_std` capable** — `tpt-mime-pure` works without the standard library
  (its `std` feature is default; disable it via `default-features = false`).

## Command-line tool

Prefer a binary over a library? [`tpt-cli`](./tpt-cli) wraps all five parsers behind a
single `tpt` executable:

```bash
cargo install --path tpt-cli

tpt cron "0 9 * * 1-5" --next   # describe a schedule (and show the next run)
tpt mime file.bin                # detect type from magic bytes (falls back to extension)
tpt geojson map.geojson          # validate a GeoJSON file
tpt logfmt 'level=info msg="hi"' # parse a logfmt line
tpt jsonl data.jsonl             # count records in a JSON Lines file
```

`tpt-cli` is **not** published to crates.io — it is a convenience front-end for the
workspace.

## Workspace & contributing

- This is a Cargo workspace (`resolver = "2"`, edition 2021, MSRV 1.71). Shared
   metadata lives in the root `Cargo.toml` via `workspace = true`; member crates
   must not hardcode it. MSRV is **1.71**.
- See [`AGENTS.md`](./AGENTS.md) for the exact verify/test/publish commands
  (`cargo fmt` / `cargo clippy -D warnings` / `cargo test`), and per-crate
  publishing on `v*.*.*` tags.
- [`CONTRIBUTING.md`](./CONTRIBUTING.md) documents how to set up, the CI
  sequence to run locally, and the project conventions.
- Each crate ships a runnable example under its `examples/` directory (run with
  `cargo run -p <crate> --example <name>`).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
