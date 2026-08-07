# Changelog

All notable changes to this crate are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-08-08

Initial published release.

### Added

- Strict GeoJSON parser: `parse` / `parse_reader` producing `GeoJson`, `Feature`,
  `FeatureCollection`, `Geometry`, and `Position` with a validation pass.
- Validation: coordinate arrays must have 2 or 3 elements, polygon rings must be
  closed (first == last) with at least 4 positions, coordinates must lie within
  longitude ∈ [-180, 180] and latitude ∈ [-90, 90], and `bbox` must be an even-length
  array of at least 4.
- `GeoError` with a dot/bracket path into the structure.
- `to_json` serialization back to valid GeoJSON, preserving `bbox` and foreign members.

### Fixed

- `Position::new` validates length, so `Position::longitude`/`latitude` can no longer
  panic on a user-constructed too-short vector.
