# Changelog

All notable changes to this crate are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `detect_all` returning a `MimeMatches` iterator over every signature that matched,
  most specific first, for inspecting ambiguous input. Allocation-free, yielding at
  most `MAX_MATCHES` items.
- ZIP entry-name inspection distinguishing `Docx`, `Xlsx`, `Pptx` and `Jar` from a
  plain `Zip`, with `Zip` kept as the fallback.
- Caveat in the `detect` documentation and in `README.md` about false positives from
  short signatures (`BM`, `MZ`, `1F 8B`, `00 00 01 00`, `CA FE BA BE`) on untrusted input.

### Fixed

- Zstandard magic bytes were wrong (`28 4D 18 09`); the frame magic `0xFD2FB528` is
  stored little-endian as `28 B5 2F FD`, so no real `.zst` file was ever detected.
- EBML `DocType` detection now parses the header structurally, decoding multi-byte
  VINT element sizes and walking child elements instead of masking the size with
  `& 0x7F` and matching `42 82` anywhere in the first 512 bytes. An EBML file with no
  recognised `DocType` in range returns `None` instead of being forced to `Mkv`.
- `detect_file` now reads in a loop up to the 8 KB cap instead of issuing a single
  `read`, so offset signatures (`ustar` at byte 257, a late EBML `DocType`) are no
  longer missed on short reads.

### Changed

- `detect_by_extension` maps `docx`, `xlsx`, `pptx` and `jar` to the new dedicated
  variants instead of `Zip`.

## [0.2.0] - 2026-08-08

### Added

- `Zstd`, `Xz`, `Woff`, `Woff2`, and `JavaClass` MIME types, detected via magic bytes
  and mapped from their extensions in `detect_by_extension`.

### Changed

- `detect_file` now reads up to 8 KB from disk (previously 512 bytes).

## [0.1.0] - 2026-07-17

Initial release.

### Added

- `MimeType` enum (zero-dependency, `#[non_exhaustive]`) with `as_str` and `extension`.
- `detect` for magic-byte sniffing and `detect_by_extension` for extension fallback.
- `detect_file` (behind the `std` feature; `no_std` compatible otherwise) reading up
  to 512 bytes from disk.
- ISO-BMFF `ftyp` brand inspection disambiguating MP4 / MOV / 3GP / HEIC / HEIF / AVIF,
  and EBML `DocType` inspection separating WebM from Matroska.
