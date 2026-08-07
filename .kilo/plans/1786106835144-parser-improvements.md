# tpt-data-parsers — Additional Improvements (2026-08-08 review)

## Context
All five crates are implemented and the workspace is pre-publish. The user asked for
improvement suggestions grounded in the actual code. I reviewed every `src/lib.rs`, the
workspace `Cargo.toml`/`ci.yml`, the README/spec, and `TODO.md`. The user's `TODO.md`
already covers Phases 11–15 (doc rot, discoverability, automation, v0.3.0, `tpt-cli`).

This plan records (a) what is **already covered**, (b) **new tasks to append to
`TODO.md`** (the user explicitly asked to add uncovered ones), and (c) **one correction**
to an item that is falsely marked done.

Do **not** touch the stale duplicate `TODO 1260713.md` (per user instruction).

## A. Already covered by existing TODO phases (no action needed here)
- MSRV doc drift (AGENTS/README say 1.70 vs Cargo/CI 1.71) → Phase 11 line 130.
- Missing `CONTRIBUTING.md` → Phase 11 line 129.
- docs.rs feature metadata, CI badge, "vs alternatives" blurbs, benchmarks, fuzzing,
  `tpt-cli`, v0.3.0 writers/iterators → Phases 12–15.

## B. New tasks to append to TODO.md
Append a short phase (e.g. "Phase 16: Parser hardening (2026-08-08)") with:

1. **GeoJSON strict coordinate-range validation** (`tpt-geo-geojson/src/lib.rs`,
   `parse_position` ~L491). The spec sells "strict validation for malformed GIS data",
   but `parse_position` accepts e.g. `[1000.0, 2000.0]`. Add a check that
   longitude ∈ [-180, 180] and latitude ∈ [-90, 90], returning
   `GeoErrorKind::MalformedCoordinates`. Non-breaking (only rejects previously-accepted
   invalid input). Add a unit test for an out-of-range point.
   - Optional: gate behind a `strict` feature if a permissive default is desired;
     recommend default-on given the spec's positioning.

2. **GeoJSON bbox length validation** (`collect_extra` ~L286). `bbox` is taken as-is via
   `as_f64` without checking RFC 7946's "even length, ≥ 4" rule. Change `collect_extra`
   to return `Result<(Option<Vec<f64>>, HashMap<...>), GeoError>` (or validate in the
   callers) and reject malformed bbox. Add a test for an odd-length bbox.

3. **GeoJSON dead code in `MultiLineString`** (`parse_geometry` ~L428-436). The arm builds
   `line_arr` then discards it with `let _ = line_arr;`; `parse_position_array` already
   validates array-ness. Remove `line_arr` and the `let _ =` line. (Currently silences
   an unused-var lint; clean it up so it can't mask real issues later.)

4. **Reconcile `tpt-mime-pure` doc vs spec** (`src/lib.rs` `detect` doc ~L151). Doc says
   "first 512 bytes" while `docs/spec.txt` promises "first 8KB". 512 covers all current
   signatures, so either bump `detect_file`'s read buffer to 8 KB to match the spec, or
   correct the doc comment to say 512. Recommend aligning the doc comment to the actual
   behaviour and noting the spec's 8 KB as the intended ceiling.

5. **Publish version decision** (workspace `Cargo.toml` L12 + Phase 6 L94-95). The
   v0.2.0 features (chrono `next_after`, geo serialize/bbox, jsonl writer) from Phase 10
   are already merged, but Phase 6 still plans to tag/publish `v0.1.0`. Decide and update
   the plan: tag `v0.2.0` (and bump `version.workspace` to `0.2.0`) since these features
   ship before first publish. AGENTS.md already instructs bumping version before publish.

## C. Correction to existing TODO (false-positive)
- **Phase 6 line 93** is checked `[x] Write CHANGELOG.md for each crate (0.1.0 entry)`,
  but no `CHANGELOG.md` exists in any crate (verified: `Get-ChildItem *CHANGELOG*` →
  none). This item is **not actually done**. Add a task to create a `CHANGELOG.md` per
  crate (Keep a Changelog format) covering the 0.1.0 baseline plus the Phase 7 fixes and
  Phase 10 features, and uncheck line 93 until created. (This overlaps with the version
  decision in B5.)

## D. Out of scope / noted-but-not-adding
- `tpt-jsonl-stream` simd path allocates a per-line `Vec<u8>` (`trimmed.as_bytes().to_vec()`,
  L112) — inherent to `simd_json` needing `&mut`; only a doc note, not a task.
- logfmt whitespace handling only skips spaces (not tabs) — matches logfmt spec; not a task.

## Validation
After implementing B1–B3: `cargo test -p tpt-geo-geojson`,
`cargo clippy -p tpt-geo-geojson -- -D warnings`.
After B4: `cargo test -p tpt-mime-pure`.
After B5/C: `cargo test --workspace --all-features`, `cargo fmt --all -- --check`,
and confirm `git status` shows the new `CHANGELOG.md` files.
