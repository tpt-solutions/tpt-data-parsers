# tpt-data-parsers — Build Checklist

## Phase 0: Workspace Scaffold
- [x] Create workspace `Cargo.toml`
- [x] Create `LICENSE-MIT`
- [x] Create `LICENSE-APACHE`
- [x] Create workspace `README.md`
- [x] Create `.github/workflows/ci.yml`
- [x] Create `.github/workflows/publish.yml`
- [x] Create `TODO.md` in project root

## Phase 1: tpt-logfmt-parse (no deps)
- [x] Scaffold `tpt-logfmt-parse/Cargo.toml`
- [x] Implement hand-rolled parser in `src/lib.rs`
- [x] Implement `LogfmtParser<'a>` zero-copy iterator
- [x] Implement `parse_to_map()` convenience function
- [x] Write `LogfmtError` type
- [x] Write unit tests (basic, quoted, escapes, bare keys, empty)
- [x] Write `tests/parse.rs` integration test
- [x] Write `tpt-logfmt-parse/README.md`
- [x] Add `#![doc = include_str!("../README.md")]` + `#![warn(missing_docs)]`
- [x] Verify `cargo test -p tpt-logfmt-parse` passes
- [x] Verify `cargo clippy -p tpt-logfmt-parse -- -D warnings` passes

## Phase 2: tpt-cron-parse (no deps)
- [x] Scaffold `tpt-cron-parse/Cargo.toml`
- [x] Implement `CronField` enum + parser
- [x] Implement `CronExpr::parse()` for 5-field and 6-field
- [x] Implement `CronExpr::to_human_readable()`
- [x] Write `CronError` type with position + field + expected/found
- [x] Write unit tests (valid expressions, error cases, human-readable)
- [x] Write `tests/cron.rs` integration test
- [x] Write `tpt-cron-parse/README.md`
- [x] Add `#![doc = include_str!("../README.md")]` + `#![warn(missing_docs)]`
- [x] Verify `cargo test -p tpt-cron-parse` passes
- [x] Verify `cargo clippy -p tpt-cron-parse -- -D warnings` passes

## Phase 3: tpt-mime-pure (no deps, no_std)
- [x] Scaffold `tpt-mime-pure/Cargo.toml` with `std` feature flag
- [x] Define `MimeType` enum with `#[non_exhaustive]`
- [x] Implement magic byte table (20 signatures)
- [x] Implement `detect(bytes: &[u8]) -> Option<MimeType>`
- [x] Implement `detect_by_extension(ext: &str) -> Option<MimeType>`
- [x] Implement `detect_file()` behind `#[cfg(feature = "std")]`
- [x] Implement `MimeType::as_str()` and `MimeType::extension()`
- [x] Write unit tests (each magic byte signature, extension fallback)
- [x] Write `tests/mime.rs` integration test
- [x] Verify `cargo test -p tpt-mime-pure --no-default-features` (no_std)
- [x] Write `tpt-mime-pure/README.md`
- [x] Add `#![doc = include_str!("../README.md")]` + `#![warn(missing_docs)]`
- [x] Verify `cargo test -p tpt-mime-pure` passes
- [x] Verify `cargo clippy -p tpt-mime-pure -- -D warnings` passes

## Phase 4: tpt-jsonl-stream (deps: serde_json)
- [x] Scaffold `tpt-jsonl-stream/Cargo.toml` with `simd` feature
- [x] Implement `JsonlReader<R: BufRead>` struct
- [x] Implement `Iterator` for `JsonlReader`
- [x] Implement `JsonlError` with line number
- [x] Wire `simd` feature to use `simd_json::from_slice`
- [x] Add `parse_jsonl()` free function
- [x] Create `tests/fixtures/sample.jsonl` (100-line fixture)
- [x] Write unit tests (empty, single, multiline, malformed with correct line)
- [x] Write `tests/streams.rs` integration test
- [x] Write `tpt-jsonl-stream/README.md`
- [x] Add `#![doc = include_str!("../README.md")]` + `#![warn(missing_docs)]`
- [x] Verify `cargo test -p tpt-jsonl-stream` passes
- [x] Verify `cargo clippy -p tpt-jsonl-stream -- -D warnings` passes

## Phase 5: tpt-geo-geojson (deps: serde, serde_json)
- [x] Scaffold `tpt-geo-geojson/Cargo.toml`
- [x] Define type hierarchy: `GeoJson`, `Feature`, `FeatureCollection`, `Geometry`, `Position`
- [x] Implement serde deserialization for all types
- [x] Implement validation pass (coordinate depth, polygon ring closure)
- [x] Implement `GeoError` with path string
- [x] Implement `parse(input: &str) -> Result<GeoJson, GeoError>`
- [x] Implement `parse_reader<R: Read>()` variant
- [x] Create `tests/fixtures/valid.geojson`
- [x] Create `tests/fixtures/malformed_coords.geojson`
- [x] Create `tests/fixtures/unclosed_polygon.geojson`
- [x] Write unit tests (all geometry types, error paths)
- [x] Write `tests/geojson.rs` integration test
- [x] Write `tpt-geo-geojson/README.md`
- [x] Add `#![doc = include_str!("../README.md")]` + `#![warn(missing_docs)]`
- [x] Verify `cargo test -p tpt-geo-geojson` passes
- [x] Verify `cargo clippy -p tpt-geo-geojson -- -D warnings` passes

## Phase 6: Final Polish (pre-publish)
- [x] `cargo test --workspace --all-features`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo fmt --all -- --check`
- [x] `cargo doc --workspace --no-deps` (no warnings)
- [x] All crates: description, keywords, categories, repository in Cargo.toml
- [x] Write `CHANGELOG.md` for each crate (0.2.0 entry) — created in Phase 16 (was falsely marked done earlier; no file existed)
- [ ] Push to GitHub, verify CI passes
- [ ] Tag `v0.2.0` to trigger publish (Phase 10 v0.2.0 features already merged; first publish is 0.2.0, not 0.1.0)

## Phase 7: Pre-publish bug fixes (from platform review)
- [x] `tpt-mime-pure`: inspect `ftyp` box brand string instead of always returning `Mp4` (fixes HEIC/HEIF/AVIF/MOV/3GP misdetection)
- [x] `tpt-mime-pure`: distinguish WebM from MKV via EBML `DocType` element (currently WebM unreachable via `detect()`)
- [x] `tpt-jsonl-stream`: stop swallowing the real `simd_json` error via `serde_json::from_str(...).unwrap_err()` re-derivation (panics on parser divergence)
- [x] `tpt-geo-geojson`: fix panic risk in `Position::longitude()`/`latitude()` on user-constructed short vectors (private field + validating constructor, or bounds-check)
- [x] `tpt-logfmt-parse`: fix stale doc comment claiming escape sequences fall back to empty slice (they don't)
- [x] `tpt-logfmt-parse`: fix non-UTF-8-aware `byte as char` handling that mangles multi-byte UTF-8 content
- [x] `tpt-cron-parse`: reject descending ranges (e.g. `5-1`) and zero step values (`*/0`) with a proper `CronError`

## Phase 8: Git/repo reconciliation
- [x] Review/merge remote PR #1 (`origin/claude/crates-io-readiness-ljshrm`) containing `hygiene.yml` and a repo-URL fix before pushing local `master`
- [x] Remove duplicate tracked file `TODO 1260715.md`
- [x] Delete or move stale `spec.txt` (superseded by README docs) into `docs/`

## Phase 9: Adoption & usability improvements
- [x] Add `examples/` directory with a runnable example per crate (crontab line, `.jsonl` stream, GeoJSON validation, logfmt line, file-type detection)
- [x] Add a "which crate do I need" decision guide to root `README.md`
- [x] Add docs.rs badges/links to root and per-crate READMEs
- [x] Add MSRV (1.70) verification job to CI
- [x] Add macOS to CI matrix
- [x] Add `CONTRIBUTING.md`, issue templates, and PR template
- [x] Add Dependabot/Renovate config for `serde`/`serde_json`/`simd-json`

## Phase 10: v0.2.0 feature candidates (post-publish, non-blocking)
- [x] `tpt-cron-parse`: "next run time" computation (added behind optional `chrono` feature; `next_after` with cron day-of-month/day-of-week OR rule)
- [x] `tpt-geo-geojson`: serialization back to valid GeoJSON text (Feature/FeatureCollection now emit `type`; `to_json` helper added)
- [x] `tpt-geo-geojson`: `bbox` field support + foreign-member passthrough on Feature/FeatureCollection
- [x] `tpt-geo-geojson`: removed unused `GeoErrorKind::InvalidCrs` variant
- [x] `tpt-jsonl-stream`: streaming writer (`JsonlWriter` + `write_jsonl` helper)
- [x] Evaluate optional serde support: kept `MimeType`/`CronExpr`/`CronField` dep-free (zero-dependency is a hard crate convention); `tpt-geo-geojson` already derives `Serialize`

## Phase 11: Doc rot & bug fixes (from 2026-08-08 platform review)
- [ ] Write missing `CONTRIBUTING.md` (README.md links to it and Phase 9 marked it done, but it was never created)
- [ ] Fix stale MSRV references: README.md and AGENTS.md say "MSRV 1.70"; root `Cargo.toml`/CI `msrv` job actually use 1.71

## Phase 12: Adoption & discoverability
- [ ] Add `[package.metadata.docs.rs] all-features = true` to `tpt-cron-parse/Cargo.toml` and `tpt-jsonl-stream/Cargo.toml` so docs.rs shows `next_after` and the `simd` feature
- [ ] Add a CI status badge (ci.yml) to root `README.md` — works pre-publish, unlike the crates.io/docs.rs badges
- [ ] Add a short "vs. alternatives" line to each crate `README.md` (why this crate over `cron`/`geojson`/`mime_guess`/etc.)

## Phase 13: Automation & quality scaffolding
- [ ] Add Criterion benchmarks (`benches/`) to all 5 crates to substantiate the "zero-allocation/ultra-fast" claims
- [ ] Add `cargo-fuzz` targets for all 5 crates + a nightly, non-blocking CI smoke-test job
- [ ] Manual/deferred: add Dependabot auto-merge workflow for passing patch-level bumps
- [ ] Manual/deferred: merge open Dependabot PRs (`serde` 1.0.229, `serde_json` 1.0.151)
- [ ] Manual/deferred: delete stale merged remote branches (`claude/crates-io-publish-check-cp1o6v`, `claude/crates-io-readiness-ljshrm`)

## Phase 14: v0.3.0 feature candidates
- [ ] `tpt-logfmt-parse`: add `write_logfmt` — the only crate in the workspace without a writer (mirrors `JsonlWriter`/`to_json`)
- [ ] `tpt-cron-parse`: add `CronExpr::upcoming` iterator for "next N runs" (builds on existing `next_after`, same `chrono` feature gate)
- [ ] `tpt-mime-pure`: add Zstandard, XZ, WOFF/WOFF2, and Java `.class` magic-byte signatures

## Phase 15: `tpt-cli` (new workspace member)
- [ ] Scaffold `tpt-cli` binary crate depending on all 5 parser crates, with `cron`/`mime`/`geojson`/`jsonl`/`logfmt` subcommands via `clap`
- [ ] Write `tpt-cli/README.md`
- [ ] Add `tpt-cli` to root `README.md` crate table, marked "(CLI, not a library)"
- [ ] Deliberately leave `tpt-cli` out of `publish.yml` — publishing stays a manual decision

## Phase 16: Parser hardening (2026-08-08 review)
- [x] `tpt-geo-geojson`: strict coordinate-range validation — reject longitude ∉ [-180,180] or latitude ∉ [-90,90] in `parse_position` (spec promises "strict validation"; `[1000,2000]` is currently accepted)
- [x] `tpt-geo-geojson`: validate `bbox` length (even, ≥ 4) in `collect_extra`; return `GeoErrorKind::MalformedCoordinates` for malformed bbox (RFC 7946)
- [x] `tpt-geo-geojson`: remove dead code in `MultiLineString` arm — `line_arr`/`let _ = line_arr` is redundant (`parse_position_array` already validates array-ness)
- [x] `tpt-mime-pure`: reconcile doc vs spec — `detect` doc says "first 512 bytes" but `docs/spec.txt` promises "first 8KB"; bump `detect_file` buffer to 8 KB and update doc comments
- [x] Write `CHANGELOG.md` for each crate (Keep a Changelog format) covering the 0.2.0 baseline, Phase 7 fixes, and Phase 10 features (Phase 6 item 93 was falsely marked done)
- [x] Bump workspace `version.workspace` to `0.2.0` (Phase 10 features ship before first publish; AGENTS.md requires bumping before publish)
