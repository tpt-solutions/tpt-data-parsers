# Changelog

All notable changes to this crate are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `parse_to_pairs` returning `Vec<(String, String)>` — an order-preserving,
  duplicate-preserving sibling of `parse_to_map` (which keeps only the last value
  for a repeated key).
- `LogfmtLinesReader<R: BufRead>` (plus the `parse_logfmt_lines` constructor) for
  multi-line logfmt sources such as Heroku router logs. Each iteration yields one
  line's `Vec<(String, String)>`; blank lines are skipped and errors carry the
  1-based line number via the new `LogfmtLinesError` / `LogfmtLinesErrorKind`.
- Quoted keys are now accepted by the parser (`"odd key"=value`), so any key the
  writer has to quote can be read back.
- `\r` and `\xNN` escape sequences are decoded inside quoted tokens.

### Changed

- **Breaking:** `LogfmtParser` now yields `(Cow<'_, str>, Cow<'_, str>)` instead of
  `(&str, &str)`. Tokens still borrow from the input; a copy is made only when a
  token contains escape sequences that must be decoded.
- **Breaking:** `write_logfmt` is now generic over `AsRef<str>` keys and values, so
  it accepts `(String, String)` pairs as well as `(&str, &str)`.
- `parse_to_map` is implemented on top of `LogfmtParser`, so the two APIs share a
  single scanning and escape-decoding implementation and cannot drift apart.

### Fixed

- `LogfmtParser` no longer loops forever on an `expected key` error: the offending
  token is consumed before the error is returned, so callers that log and continue
  make progress instead of hanging.
- `\r`, `\n`, and `\t` are treated as token separators by both the parser and the
  writer. `parse_to_map("a=1\r\n")` now yields `{"a": "1"}` instead of a value with
  a trailing CRLF, and `a=1\tb=2` parses as two pairs instead of erroring.
- `write_logfmt` / `format_pair` now round-trip through the parser: keys and values
  containing spaces, quotes, backslashes, `=`, or control characters are quoted, and
  control characters are escaped (`\n`, `\r`, `\t`, `\xNN`) so a value with a newline
  can no longer break the single-line logfmt contract.
- `LogfmtParser` and `parse_to_map` now return identical, decoded values for the same
  input; the iterator previously returned the raw slice including backslashes.

## [0.2.0] - 2026-08-08

Initial published release.

### Added

- Zero-copy, hand-rolled logfmt parser: `LogfmtParser` streaming iterator yielding
  `(&str, &str)` pairs borrowed from the input (no regex, no intermediate allocation).
- `parse_to_map` convenience function returning an owned `HashMap<String, String>`,
  with quoted-string escape sequences (`\"`, `\\`, `\n`, `\t`) resolved.
- `LogfmtError` carrying the byte offset and a human-readable message.

### Fixed

- Multi-byte UTF-8 content is now parsed correctly (the previous byte-oriented
  handling mangled non-ASCII keys/values).
- Stale doc comment claiming escape sequences fall back to an empty slice (they do not).
