# Changelog

All notable changes to this crate are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `JsonlReader::into_typed::<T>()` returning `TypedJsonlReader`, an iterator of
  `Result<T, JsonlError>` that deserializes each record into any `DeserializeOwned`
  type while keeping 1-based line numbers on both parse and type errors.
- `JsonlReader::with_max_line_length` / `JsonlReader::max_line_length` and the
  `DEFAULT_MAX_LINE_LENGTH` constant (16 MiB) to bound per-line buffering.
- `JsonlErrorKind::LineTooLong { limit }` for lines that exceed that cap.
- `AsyncJsonlReader` (under the `tokio` feature), an async JSON Lines reader over
  `tokio`'s `AsyncBufRead` with the same line-length cap and error handling as the
  synchronous reader.

### Fixed

- Unbounded per-line buffering: the reader no longer grows a buffer without limit for
  newline-free input. Lines longer than the configured maximum are discarded without
  being buffered, reported as `JsonlErrorKind::LineTooLong`, and the stream resumes at
  the next line.
- `JsonlWriter::write` no longer leaves a partial record in the output when
  serialization fails: the line is serialized into an internal buffer and written in a
  single call, so a failed write emits nothing and later writes stay well-formed.
- The `simd` feature is no longer a pessimization: lines are parsed in place from the
  reader's own buffer instead of being copied into a fresh `Vec` per line, and a
  `simd-json` rejection is surfaced directly instead of triggering a second parse of a
  buffer that `simd-json` may already have mutated.
- Blank-line detection now follows the JSON Lines convention: only empty lines and
  lines made solely of ASCII whitespace are skipped, so a line containing a
  non-breaking space is parsed rather than silently dropped. Lines are no longer
  trimmed before parsing, so `serde_json` error line/column offsets refer to the
  original line.

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
