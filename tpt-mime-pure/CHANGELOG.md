# Changelog

All notable changes to this crate are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-08-08

Initial published release.

### Added

- `MimeType` enum (zero-dependency, `#[non_exhaustive]`) with `as_str` and `extension`.
- `detect` for magic-byte sniffing and `detect_by_extension` for extension fallback.
- `detect_file` (behind the `std` feature; `no_std` compatible otherwise) reading up
  to 8 KB from disk.
- ISO-BMFF `ftyp` brand inspection disambiguating MP4 / MOV / 3GP / HEIC / HEIF / AVIF,
  and EBML `DocType` inspection separating WebM from Matroska.

### Fixed

- `detect` previously always returned `Mp4` for any ISO-BMFF file; it now inspects the
  major-brand string so HEIC/HEIF/AVIF/MOV/3GP are detected correctly.
- WebM was previously unreachable via `detect` (overlapped with Matroska); it is now
  distinguished via the EBML `DocType` element.
