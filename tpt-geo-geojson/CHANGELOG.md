# Changelog

All notable changes to this crate are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `GeoJson::validate`, `Geometry::validate`, and `Feature::validate` re-run the full
  validation pass on a hand-constructed value, not just during `parse`.
- `Geometry::bounding_box()` computes the bounding box from a geometry's own positions.
- `Geometry::contains(&Position)` point-in-polygon test (ray-casting, hole-aware) for
  polygons.
- `BoundingBox::to_vec()` to recover the underlying coordinate list.
- Bare top-level `Geometry` objects now collect and round-trip `bbox` and foreign members.
- `Deserialize` for `Position`, `Geometry`, `GeometryObject`, `Feature`,
  `FeatureCollection`, and `GeoJson`, routed through the same validation `parse` uses —
  hand-constructed values (e.g. via `serde_json::from_value` or embedding these types in
  your own `#[derive(Deserialize)]` struct) now get the same construction-time
  guarantees as `parse`, closing the gap left when only `validate()` shipped earlier.

### Fixed

- Ring closure uses exact equality (including the altitude component) instead of an
  `f64::EPSILON` tolerance, matching RFC 7946 (previously both false-positives and
  false-negatives were possible near large coordinates or the origin).
- `bbox` is now validated: every element must be a finite number, length must be even and
  `>= 4` (2D) or `>= 6` (3D), and longitude/latitude are range-checked; malformed bbox is
  rejected instead of silently coerced.
- `foreign_members` uses a `BTreeMap` so `to_json` emits deterministic key order.
- `GeoError` paths no longer carry a leading stray dot (e.g. `coordinates`,
  `geometry.coordinates`).
- Strict coordinate-range validation rejects longitude ∉ [-180, 180] or
  latitude ∉ [-90, 90] during parsing.

## [0.2.0] - 2026-08-08

### Added

- `bbox` validation during parsing: must be a numeric array of even length ≥ 4;
  malformed `bbox` is now rejected instead of silently accepted.
- Coordinate range validation during parsing: positions with longitude outside
  [-180, 180] or latitude outside [-90, 90] are now rejected.

## [0.1.0] - 2026-07-17

Initial release.

### Added

- Strict GeoJSON parser: `parse` / `parse_reader` producing `GeoJson`, `Feature`,
  `FeatureCollection`, `Geometry`, and `Position` with a validation pass.
- Validation: coordinate arrays must have 2 or 3 elements, and polygon rings must be
  closed (first == last) with at least 4 positions.
- `GeoError` with a dot/bracket path into the structure.
- `to_json` serialization back to valid GeoJSON, preserving `bbox` and foreign members.
- `Position::new` validates length, so `Position::longitude`/`latitude` can no longer
  panic on a user-constructed too-short vector.
