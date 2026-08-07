# Changelog

All notable changes to this crate are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-08-08

Initial published release.

### Added

- `JsonlReader` streaming `Iterator` over `serde_json::Value`, one value per non-empty
  line, with 1-based line numbers on parse errors; `parse_jsonl` convenience wrapper.
- `JsonlWriter` and `write_jsonl` for streaming JSON Lines output.
- Optional `simd` feature wiring `simd-json` for faster parsing on compatible CPUs.

### Fixed

- The `simd` path no longer panics on parser divergence: a `simd-json` rejection is
  re-checked with `serde_json`, surfacing the precise error (or the value) rather than
  calling `.unwrap_err()` on a re-derived parse.
