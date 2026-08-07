# Changelog

All notable changes to this crate are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
