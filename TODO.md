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
- [x] Push to GitHub, verify CI passes
- [x] Tag `v0.2.0` to trigger publish (Phase 10 v0.2.0 features already merged; first publish is 0.2.0, not 0.1.0)

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
- [x] Write missing `CONTRIBUTING.md` (README.md links to it and Phase 9 marked it done, but it was never created)
- [x] Fix stale MSRV references: README.md and AGENTS.md say "MSRV 1.70"; root `Cargo.toml`/CI `msrv` job actually use 1.71 (verify: all docs now state 1.71)

## Phase 12: Adoption & discoverability
- [x] Add `[package.metadata.docs.rs] all-features = true` to `tpt-cron-parse/Cargo.toml` and `tpt-jsonl-stream/Cargo.toml` so docs.rs shows `next_after` and the `simd` feature
- [x] Add a CI status badge (ci.yml) to root `README.md` — works pre-publish, unlike the crates.io/docs.rs badges
- [x] Add a short "vs. alternatives" line to each crate `README.md` (why this crate over `cron`/`geojson`/`mime_guess`/etc.)

## Phase 13: Automation & quality scaffolding
- [x] Add Criterion benchmarks (`benches/`) to all 5 crates to substantiate the "zero-allocation/ultra-fast" claims
- [x] Add `cargo-fuzz` targets for all 5 crates + a nightly, non-blocking CI smoke-test job
- [x] Manual/deferred: add Dependabot auto-merge workflow for passing patch-level bumps (`.github/workflows/dependabot-auto-merge.yml`)
- [x] Manual/deferred: merge open Dependabot PRs (`serde` 1.0.229, `serde_json` 1.0.151) — merged into `master` and pushed
- [x] Manual/deferred: delete stale merged remote branches (`claude/crates-io-publish-check-cp1o6v`, `claude/crates-io-readiness-ljshrm`) — deleted from `origin`

## Phase 14: v0.3.0 feature candidates
- [x] `tpt-logfmt-parse`: add `write_logfmt` — the only crate in the workspace without a writer (mirrors `JsonlWriter`/`to_json`)
- [x] `tpt-cron-parse`: add `CronExpr::upcoming` iterator for "next N runs" (builds on existing `next_after`, same `chrono` feature gate)
- [x] `tpt-mime-pure`: add Zstandard, XZ, WOFF/WOFF2, and Java `.class` magic-byte signatures

## Phase 15: `tpt-cli` (new workspace member)
- [x] Scaffold `tpt-cli` binary crate depending on all 5 parser crates, with `cron`/`mime`/`geojson`/`jsonl`/`logfmt` subcommands via `clap`
- [x] Write `tpt-cli/README.md`
- [x] Add `tpt-cli` to root `README.md` crate table, marked "(CLI, not a library)"
- [x] Deliberately leave `tpt-cli` out of `publish.yml` — publishing stays a manual decision

## Phase 16: Parser hardening (2026-08-08 review)
- [x] `tpt-geo-geojson`: strict coordinate-range validation — reject longitude ∉ [-180,180] or latitude ∉ [-90,90] in `parse_position` (spec promises "strict validation"; `[1000,2000]` is currently accepted)
- [x] `tpt-geo-geojson`: validate `bbox` length (even, ≥ 4) in `collect_extra`; return `GeoErrorKind::MalformedCoordinates` for malformed bbox (RFC 7946)
- [x] `tpt-geo-geojson`: remove dead code in `MultiLineString` arm — `line_arr`/`let _ = line_arr` is redundant (`parse_position_array` already validates array-ness)
- [x] `tpt-mime-pure`: reconcile doc vs spec — `detect` doc says "first 512 bytes" but `docs/spec.txt` promises "first 8KB"; bump `detect_file` buffer to 8 KB and update doc comments
- [x] Write `CHANGELOG.md` for each crate (Keep a Changelog format) covering the 0.2.0 baseline, Phase 7 fixes, and Phase 10 features (Phase 6 item 93 was falsely marked done)
- [x] Bump workspace `version.workspace` to `0.2.0` (Phase 10 features ship before first publish; AGENTS.md requires bumping before publish)

## Phase 17: Bug fixes & doc/changelog hygiene (2026-08-08 review)
- [x] `tpt-mime-pure`: fix incorrect Zstandard magic-byte signature — `detect()` checks for `[0x28, 0x4D, 0x18, 0x09]` (`src/lib.rs:331`, and the test at `:543` encodes the same wrong bytes), but real `.zst` files start with `28 B5 2F FD` (magic number `0xFD2FB528`, little-endian). Every real zstd-compressed file is currently misdetected as unknown type.
- [x] `tpt-logfmt-parse`: fix `write_logfmt`/`format_pair` not escaping control characters — `needs_quoting`/`quote_token` (`src/lib.rs:302-321`) only trigger/escape on space, `"`, `\`, `=`; a value containing a raw `\n` or `\t` is either emitted completely unquoted (breaking the single-line invariant) or copied verbatim inside quotes instead of being encoded as `\n`/`\t` — asymmetric with the parser, which *does* decode `\n`/`\t` escapes (`:234-235`). Round-tripping such a value through `write_logfmt` → `parse_to_map` corrupts data; unescaped raw newlines from attacker-controlled strings are also a log-injection (CWE-117) vector.
- [x] `tpt-cron-parse`: add per-field range validation — `parse_u8`/`parse_item` (`src/lib.rs:332-413`) accept any `u8` (0-255) with no bounds check, so e.g. `"0 99 * * *"` parses successfully; `to_human_readable()` then prints nonsense, and under the `chrono` feature `next_after`/`upcoming` silently run their full bounded ~4-year/2.1M-minute search (the search itself is correctly bounded, not infinite — but a malformed field should fail fast with a `CronError` instead of paying that cost). No existing test covers rejection of out-of-range field values.
- [x] `tpt-geo-geojson`: `bbox` on a bare top-level `Geometry` (not wrapped in a `Feature`/`FeatureCollection`) is silently discarded — `parse_geometry` never calls `collect_extra`, so `to_json` won't reproduce it. RFC 7946 permits `bbox` on any GeoJSON object; today only Feature/FeatureCollection round-trip it.
- [x] Backfill `CHANGELOG.md` entries — `write_logfmt`, `CronExpr::upcoming`, and the new mime signatures (Zstd/XZ/WOFF/WOFF2/JavaClass) shipped but have zero mentions in any crate's `CHANGELOG.md` (all stop at `[0.2.0]`), despite `TODO.md` Phase 14 labeling them "v0.3.0 feature candidates."

## Phase 18: CI & supply-chain automation
- [x] Add a `cargo-deny` or `cargo-audit` CI job — nothing today checks for vulnerable dependencies or license drift; `hygiene.yml` only checks commit metadata, not supply-chain health.
- [x] Add a `cargo-semver-checks` gate before/within `publish.yml` — a breaking change could currently be published under a patch/minor bump with nothing catching it across 5 independently-versioned public crates.
- [x] Add a `cargo doc --no-deps` dry-run CI job per crate so a broken `#![doc = include_str!("../README.md")]` or intra-doc link is caught before it breaks the docs.rs build post-publish.

## Phase 19: Adoption & CLI ergonomics
- [x] `tpt-cli`: accept stdin (`-` or piped input) for `mime`/`geojson`/`jsonl`, not just file paths — breaks the standard Unix pipeline idiom (`cat f | tpt jsonl`) that a CLI wrapping stream parsers should support.
- [x] `tpt-cli`: add a `--json`/`--format` output flag — every subcommand currently `println!`s ad-hoc human text with no machine-readable mode, which undercuts scripting/CI use.
- [x] `tpt-cli`: add shell-completion generation (`clap_complete`).
- [x] `tpt-cli`: distribute prebuilt binaries (e.g. `cargo-dist` release workflow) — today the only install path is cloning the full workspace and `cargo install --path tpt-cli`, a real barrier for non-Rust-dev users who just want a `mime`/`cron` utility.
- [x] Add one example composing 2+ crates together (e.g. `tpt-mime-pure::detect` routing bytes to the matching parser) — all current per-crate examples are minimal single-crate demos; a composed example is the natural next step given the README already sells a "which crate do I need" decision path.
- [x] `tpt-cli`: add a `tpt sniff <path>` command that runs mime detection first and auto-dispatches to the matching parser subcommand — directly composes the existing crates with zero new dependencies.

## Phase 20: v0.4.0 feature candidates (non-blocking backlog)
- [x] `tpt-cron-parse`: `@daily`/`@hourly`/`@weekly`/`@monthly`/`@yearly`/`@reboot` shorthand aliases (pure string preprocessing, zero new deps).
- [x] `tpt-cron-parse`: named months/days-of-week (`JAN`, `MON`, ...) in addition to numeric fields (zero new deps).
- [x] `tpt-cron-parse`: timezone-aware `next_after_tz` variant gated behind a new optional `chrono-tz` feature (mirrors the existing `chrono` feature pattern; new optional dep, not mandatory).
- [x] `tpt-mime-pure`: inspect ZIP-internal entry names to distinguish docx/xlsx/pptx/jar from plain ZIP (same technique already used for the ftyp/EBML brand checks; zero new deps).
- [x] `tpt-mime-pure`: `detect_all(bytes) -> Vec<MimeType>` to surface ambiguous/overlapping signature matches instead of silently picking the first (zero new deps).
- [x] `tpt-jsonl-stream`: typed iterator (`JsonlReader::into_typed::<T>()` or similar) so callers stop manually calling `serde_json::from_value` on every yielded `Value` (zero new deps, symmetric with the existing generic writer).
- [x] `tpt-jsonl-stream`: optional `tokio` feature for `AsyncBufRead` streaming (new optional dep, gated like `simd`).
- [x] `tpt-geo-geojson`: `Geometry::bounding_box()` computed from a geometry's own positions (zero new deps).
- [x] `tpt-geo-geojson`: point-in-polygon (`Geometry::contains(&Position)`, ray-casting, hole-aware) — natural fit given `Polygon`'s ring structure is already modeled (zero new deps).
- [x] `tpt-logfmt-parse`: `LogfmtLinesReader<R: BufRead>` mirroring `JsonlReader`'s shape for multi-line logfmt sources (Heroku router logs, etc.) — the biggest gap here since only single-line parsing exists today (zero new deps).
- [x] `tpt-logfmt-parse`: `parse_to_pairs()` as an order/duplicate-preserving sibling to `parse_to_map` (`HashMap` currently silently drops order and duplicate keys) (zero new deps).

## Phase 21: Skeptical-engineer review (code audit, 2026-08-08) — CONFIRMED BUGS

> The following were reproduced empirically (scratch harness + `cargo` runs), not just read.
> Each line is a distinct defect. Items already tracked elsewhere are NOT repeated here.

### tpt-logfmt-parse (parser correctness)
- [x] **Iterator never advances on `expected key` error → infinite loop on `for x in parser`.** `LogfmtParser::next` (`src/lib.rs:126-132`) returns `Err` without moving `pos` when the next token is `=value` or `"quoted"=1`, so every subsequent `next()` yields the identical error. Any caller that logs-and-continues hangs the process. Rename/relabel is insufficient — must consume the offending token.
- [x] **`skip_whitespace` and token breaking ignore CRLF/tabs** (`src/lib.rs:58-62, 65-74, 169-172`). `"a=1\r\n"` parses `a` = `"1\r\n"`; `"a=1\tb=2"` errors at the tab. Real log lines and CRLF files are mishandled. (Tested: `parse_to_map("a=1\r\n")` → `{"a":"1\r\n"}`.)
- [x] **`write_logfmt`/`format_pair` do NOT round-trip through the parsers** (cross-reference Phase 17 control-char item). `write_logfmt([("a b","x")])` → `"\"a b\"=x"`, which `parse_to_map` rejects (`expected key`) — the two halves of the crate are mutually incompatible. Also a value containing a raw `\n` is emitted as a literal newline, violating the "single logfmt line" contract.
- [x] **Two public APIs disagree on identical input.** `LogfmtParser` (`src/lib.rs:84-114`) returns the *raw* slice including backslashes (`say \"hi\"`), while `parse_to_map` (`src/lib.rs:216-245`) decodes escapes (`say "hi"`). Same line, two different values — `LogfmtParser` is documented as zero-copy but is effectively useless for quoted values because of this. The escape-decoding logic is duplicated instead of shared, which is how they drifted.

### tpt-cron-parse (parser correctness)
- [x] **Trailing garbage after the last field is silently accepted.** `parse` (`src/lib.rs:415-457`) validates field *count* via `split_whitespace` but never checks for trailing bytes in the final field, so `"* * * * 1garbage"` and `"* * * * *DROP TABLE"` both `parse()` successfully. Anything past the 5th/6th field is ignored. Add a trailing-input check with a `CronError`.
- [x] **`Step` over a `Range` ignores the range end.** `expand_field` (`src/lib.rs:276-296`) iterates `while v <= max` (the field max, 59/23/31/12/7) instead of `<= range_end`. `0 1-5/2 * * *` therefore fires at hours 1,3,5,7,9,…23 — not 1,3,5. `next_after`/`upcoming` inherit the wrong set. (Reproduced: `upcoming` over 12 iterations yields hours 1,3,5,7,9,11,13,15,17,19,21,23.)
- [x] **6-field `next_after` skips the current minute entirely.** `src/lib.rs:177-218` starts `min_start = after_minute + 1min` and the `same_minute` branch (`:204`) can never be true, so `"30 0 9 * * *"` after `09:00:00` returns `next day 09:00:30` (test only checks h/m/s, masking this). `*/1 * * * * *` after `09:00:00` returns `09:01:00` instead of `09:00:01`.
- [x] **Cron rejects tab-separated fields but `split_whitespace` (used for the count check) accepts them** (`src/lib.rs:419` vs `:326-330`). `"*\t*\t*\t*\t*"` is reported as 5 fields by the count check but fails parsing in the `hours` field — internally inconsistent and a spec mismatch (real crontabs use tabs).

### tpt-geo-geojson (validation / spec)
- [x] **Ring-closure test uses `f64::EPSILON`, not equality — both false-positives and false-negatives.** `parse_ring_array` (`src/lib.rs:591-592`) uses `> f64::EPSILON`. Because one ULP at `lon≈122` is ~1.4e-14 ≫ EPSILON, a ring off by one floating-point bit at large coordinates is wrongly *rejected*; near the origin, a ring off by 1e-17 is wrongly *accepted* as closed; and the altitude component is ignored entirely (a ring `[0,0,5]…[0,0,9]` passes). RFC 7946 requires exact closure; the fix is `first == last`. (Reproduced all three cases.)
- [x] **`bbox` silently coerced, not validated.** `collect_extra` (`src/lib.rs:289-310`) uses `filter_map(Value::as_f64)` and *drops* non-numeric entries with no error, so `[0,0,"junk",10,10]` and `[null,null,null,null,1,2,3,4]` both parse and round-trip with the bad members gone, and there is **no longitude/latitude range check** on bbox (unlike positions). RFC 7946 bbox can also be out of range (`[-9999,-9999,9999,9999]` accepted). Malformed-but-even-length bbox (e.g. length 6) is also accepted.
- [x] **No public `Deserialize` impls; `Geometry`/`Feature`/etc. cannot be `#[derive(Deserialize)]`d**, and **no `validate()` that re-checks a hand-constructed value** — `to_json` will happily emit *invalid* GeoJSON (NaN/Inf positions → `[null,null]`; 1-point "Polygon"; unclosed rings) because validation only runs during `parse`. Construction-time guarantees are absent.
- [x] **`foreign_members` is a `HashMap`**, so `to_json` emits non-deterministic key order across runs/machines; the "round-trip" tests only pass because they re-`parse` into another non-deterministic map. Use `BTreeMap` if stable output is claimed.
- [x] **Error `path` carries a leading stray `.`** (e.g. `.coordinates`, `.geometry.coordinates`) — `format!("{}.coordinates", "")` → `.coordinates`. Cosmetic but the docs promise "dot/bracket notation" starting at the root, not a leading dot.
- [x] **Non-conformance / silent rewrites:** `properties` is conflated with `properties: null` (input without `properties` is rewritten to `properties: null`); `Feature` with `properties: 42` (non-object, non-null) is accepted; degenerate geometries (`LineString: []`, `Polygon: []`, `MultiPolygon: []`) are accepted. None match RFC 7946.

### tpt-jsonl-stream (memory safety / perf)
- [x] **Unbounded per-line buffering (DoS).** `JsonlReader::next` (`src/lib.rs:92-147`) calls `read_line` with no line-length cap and no `max` option. A single newline-free 100 GB line allocates 100 GB resident; for untrusted `.jsonl` this is a trivial OOM. Add a configurable max line length that yields an error instead of allocating.
- [x] **`JsonlWriter::write` leaves corrupt partial output on serialize failure.** `src/lib.rs:208-219` writes the JSON *then* expects `write_all("\n")`; when `serde_json::to_writer` fails partway, the partial bytes remain in the sink and the next successful `write` appends after them, producing garbage lines (e.g. `{"ok":1}\n{{"ok":2}`). Buffer to a temp and emit atomically, or document the corruption contract.
- [x] **`simd` feature is a pessimization, not a 3× speedup.** `src/lib.rs:110-132` does `trimmed.as_bytes().to_vec()` per line (fresh heap alloc + copy) before `simd_json::from_slice`, defeating SIMD's alloc-avoidance, and on any parse error it *re-parses the same line with serde_json*. Benchmark (50k realistic lines, this machine): default ≈ 145 MB/s, simd ≈ 100 MB/s — **slower**, contradicting the README/per-crate-doc "3× SIMD throughput" claim. Either fix the copy or stop advertising 3×.
- [x] **`trim()` treats all Unicode whitespace (incl. NBSP) as blank lines**, diverging from the JSONL convention of "truly empty", and it also shifts `serde_json::Error` column offsets (no test asserts the original byte offset).

### tpt-mime-pure (detection quality / false positives)
- [x] **Several signatures are 2–4 bytes and produce frequent false positives on ordinary text.** `BM`→BMP, `MZ`→PE/exe, `00 00 01 00`→ICO, `CA FE BA BE`→JavaClass (also matches Mach-O fat binaries), `1F 8B`→Gzip. These are fine for *trusted* files but dangerous if used for upload/content-type decisions; the README sells "works wherever `file` isn't available" without any caveat. Document the false-positive risk or require longer/anchored signatures.
- [x] **EBML DocType scan is brittle.** `detect` (`src/lib.rs:266-287`) masks the VINT size with `& 0x7F` (only single-byte sizes) and matches `42 82` *anywhere* in the first 512 bytes (can match inside other elements), and **silently defaults to `Mkv` instead of returning `None` when the DocType sits past the 512-byte window** — so a real WebM whose `webm` DocType is beyond offset 512 is reported as `Mkv`, and any other EBML subtype is also forced to `Mkv`. (Reproduced: WebM with DocType at offset ~604 → detected `Mkv`.)
- [x] **`detect_file` reads only one `read()`** (`src/lib.rs:484-490`) — on pipes/filesystems that return short reads, `tar` (needs offset 257) and other offset signatures can miss. Use a loop/`read_to_end` like `tpt-cli` does.

### tpt-cli (robustness / correctness)
- [x] **`println!` panics on broken pipe.** Every subcommand prints with `println!`; piping to `head`/`less` that closes early causes `failed printing to stdout: Broken pipe` (RawMode/IO panic), a terrible CLI failure mode. Use `writeln!` to stdout with ignored errors (stderr already does this correctly).
- [x] **`Mime` and `Geojson` read the ENTIRE file into memory** (`src/main.rs:75-79`, `:94-98`) even though `tpt-mime-pure` has a 8 KB `detect_file` and `tpt-geo-geojson` has `parse_reader`. A huge input OOMs the CLI; route through the streaming/limited APIs.
- [x] **`Logfmt` subcommand prints raw unescaped slices** (`src/main.rs:103-108`) via `LogfmtParser`, so `"msg=\"say hi\""` prints `msg = say \"hi\"` (literal backslashes) — inconsistent with `parse_to_map` and unreadable. Use the decoding API.
- [x] **`cron --next` is hardcoded to UTC with no timezone flag/no mention in help**, so non-UTC users get silently wrong "next run" times.

### CI / release engineering (release.yml, ci.yml)
- [x] **`publish.yml` does not depend on `ci.yml` and has no dry-run/`--allow-dirty` guard**, so a tag can be published even if tests failed on the same commit; re-tagging an already-published version errors mid-way and leaves later crates unpublished with no rollback.
- [x] **CI only runs `--all-features`**, so the *default* (non-simd, non-chrono) code paths `cargo clippy`/`cargo test` never execute in CI. `cargo test --workspace` (default) and `cargo clippy --workspace` should also run.
- [x] **`fuzz` job is `continue-on-error` AND only builds targets (never runs them)**, so the fuzz investment catches nothing in CI. Either run a short `cargo fuzz run` or drop the pretense.
- [x] **`#![doc = include_str!("../README.md")]` is never built in CI** (`cargo doc --no-deps` dry-run absent, see Phase 18) — a broken doc include breaks the post-publish docs.rs build with no pre-check.
- [x] **`no_std` claim is never compiled for a bare-metal target in CI.** It *does* build (`cargo build -p tpt-mime-pure --no-default-features --target thumbv7em-none-eabi` succeeds) but that's unverified in CI; add a `cargo build --no-default-features --target <none>` job or a `cargo build --no-default-features` doc/fuzz check.

## Phase 21b: Corrections to earlier TODO entries (2026-08-08 audit)
- [x] **Re-verify before acting — my prior correction below was wrong; the Phase 17 Zstandard bullet is CORRECT.** `tpt-mime-pure/src/lib.rs:330-333` uses `starts_with!([0x28, 0x4D, 0x18, 0x09])`. The canonical zstd frame magic (32-bit `0xFD2FB528`) is written little-endian on disk as bytes `28 B5 2F FD`, **not** `28 4D 18 09`. So real `.zst` files ARE misdetected → the original Phase 17 item stands and should be fixed (`28 B5 2F FD`). The byte claim in Phase 17 is accurate; do not strike it.

## Phase 22: 2026-08-08 platform audit — VERIFIED BUGS (empirically reproduced)
> Findings below were each reproduced by running scratch harnesses / the built CLI;
> the checkbox only goes `[x]` once a regression test exists that fails before and
> passes after (this is what kept earlier phases from regressing).

### CRITICAL
- [x] `tpt-jsonl-stream`: `simd` feature stack-overflows on deeply nested JSON (~4 KB input at depth ~2000; default path safely caps at depth 128). Add a cheap depth pre-scan rejecting depth beyond 128 with `serde_json::Error` so both feature paths behave identically; add a `--features simd` regression test.
- [x] `tpt-jsonl-stream`: `JsonlReader::next` loops forever on a persistent I/O error (reproduced: 2000/2000 items). Add a `done` latch set on `Eof`/`Io`; return `None` afterwards; `impl FusedIterator`. Do the same for `AsyncJsonlReader`.
- [x] `tpt-cli`: 6-field `cron --next` returns a time **in the past** (`with_second(0)` truncation at `main.rs` resets the library's seconds floor). Remove the truncation so `next_after` handles seconds.

### HIGH
- [x] `tpt-cli`: `cron --next --timezone` ignores the timezone for schedule evaluation (only re-renders UTC as the zone). Wire `next_after_tz` (the library API already exists).
- [x] `tpt-cron-parse`: `to_human_readable()` silently drops the seconds field — `*/10 * * * * *`/`* * * * * *`/`30 0 9 * * *`/`0 0 9 * * *` all collapse. Thread `seconds` through; append `:SS` / "Every N seconds".
- [x] `tpt-cron-parse`: `to_human_readable()` fallback is gibberish for common exprs (`0 9 * * 1-5` → `"At 0 past 9 on * of * (1-5)"`; `"Every 1 minutes"`). Handle `Range`/`List`/DOW and an "every N hours" case; make the fallback grammatical. (`0 9 * * 1-5` is the README/example's own expression.)
- [x] `tpt-cli`: `--json` / `--format json` emits non-JSON on every error path (stderr text, exit 1). Thread `format` out of `run` and emit `{"error":..,"kind":..}` JSON on failure too; add `"ok"` to success payloads.
- [x] `tpt-cli`: `sniff` only runs MIME detection on *invalid* UTF-8 (reverse of its README); ASCII-clean tar → `unknown`. Run `detect()` unconditionally first; only fall through to text classification when it returns `None`/text-ambiguous.
- [x] `tpt-logfmt-parse`: escape set diverges from `go-logfmt` both ways — `\uXXXX`/`\b`/`\f`/`\/` are silently mis-decoded (passed through with backslashes, no error), and the writer emits `\xNN` (which go-logfmt rejects). Decode `\uXXXX`(+surrogates)/`\b`/`\f`/`\/`; encode control bytes as `\u00XX`. Decide/document unknown-escape policy.
- [x] `tpt-geo-geojson`: degenerate `LineString: []` / `[[0,0]]` accepted (RFC 7946 §3.1.4). Add a `len < 2` check in `parse` and `validate`.
- [x] `tpt-geo-geojson`: `properties` absent is conflated with `null` (breaks `parse→to_json→parse` equality) and `properties: 42` is accepted and re-emitted. Require `properties`; accept only object-or-null per RFC 7946 §3.2.
- [x] `tpt-geo-geojson`: nested-geometry `bbox`/foreign members are silently dropped *and* unvalidated (top-level bbox:"junk" rejected, one level down accepted). Make `collect_extra` run on nested geometries; preserve/validate them, or document the drop.
- [x] `tpt-mime-pure`: `tar` shadowed by 2-byte offset-0 signatures (`MZ`/`%PDF`/...) — a tar whose first member name is `MZ-report.txt` detects as `application/x-msdownload`. Reorder detection so offset-anchored/longer signatures win, or sort a specificity table.

### MEDIUM
- [x] `tpt-cron-parse`: `CronField::expand` doesn't normalise DOW `7`→`0` (`*` returns 8 values for 7 days). Document or add a normalisation variant.
- [x] `tpt-cron-parse`: leap-year search bound is 4 years but must be 8 (2096→2104 gap); `0 0 29 2 *` after 2096-03-01 → `None`. Use `8*366+1`; fix doc/README; ideally return a distinct `SearchLimitExceeded`.
- [x] `tpt-cron-parse`: DOM/DOW OR-rule diverges from Vixie when a field *starts with `*`* but isn't bare `*` (`0 0 */2 * MON` fires Saturday, should be Monday). Record a `dom_star`/`dow_star` flag at parse time.
- [x] `tpt-cron-parse`: `*/200 * * * *` accepted → "Every 200 minutes"; no `L`/`W`/`#`/`?` support (document the gap or implement `?`/`L`/`#`). Gap documented in `parse` doc; `?`/`L`/`W`/`#` rejected as parse errors.
- [x] `tpt-logfmt-parse`: bare value containing `=` is silently truncated and returned `Ok` (`sig=YWJjZA==` → `("sig","YWJjZA")`). Emit an error on `=`/`"` inside an unquoted value and consume the token.
- [x] `tpt-logfmt-parse`: `LogfmtLinesReader` has no line-length cap (jsonl sibling got one in the same phase). Mirror `with_max_line_length`/`LineTooLong`.
- [x] `tpt-jsonl-stream`: BOM on the first line makes it always fail. Strip a leading `\u{feff}` on line 0 before parsing.
- [x] `tpt-geo-geojson`: inverted-latitude bbox accepted (`[0,10,10,0]`); antimeridian bbox spans 358°. Validate `south<=north`; document/guard antimeridian.
- [x] `tpt-geo-geojson`: `to_json` emits `[null,null]` for NaN coords though `validate()` rejects them. Validate (or `to_json_validated`) before serialising.
- [x] `tpt-cli`: `logfmt` output order is nondeterministic across runs and drops duplicate keys (uses `HashMap`; `parse_to_pairs` shipped in Phase 20 and is unused here). Switch the human path to `parse_to_pairs`; carry order/duplicates in `SniffCategory::Logfmt`.
- [x] `tpt-cli`: `sniff` exits **0** on unknown while `mime` exits **1**. Pick one convention and document it.
- [x] `tpt-cli`: extension fallback fires on files with no extension (`rsplit('.')` on `exe` → `application/x-msdownload` on a 0-byte file). Use `Path::extension()`.
- [x] `tpt-cli`: `geojson` subcommand reads one syscall per byte (`parse_reader` on an unbuffered reader). Wrap in `BufReader`.
- [x] `tpt-cli`: `--json` and `--format` silently contradict; `--json` leaks a library impl detail into `--help`. Make `--json` an alias / deprecate.
- [x] `tpt-cli`: `sniff` silently truncates at 1 MiB → wrong record counts and spurious `unknown`. Stream instead of slurp, drop the trailing partial line, and (if kept) document the cap; set `"truncated": true`.
- [x] `tpt-mime-pure`: AVIF with `mif1` major brand + `avif` in compatible brands → `image/heif`. Scan the `ftyp` compatible-brands list for a more specific match.
- [x] `tpt-mime-pure`: MPEG-2.5 Layer III (`FF E3`) undetected; MP3 sync test could mask-test.
- [x] `tpt-mime-pure`: no text/charset detection at all (html/json/svg/xml/plain → `None`). Add text handling and BOM sniffing.
- [x] `tpt-mime-pure`: missing common formats (7z, RAR, bzip2, OLE2/legacy Office, Mach-O, TTF/OTF, etc.).

### LOW / DOCS
- [x] `tpt-cli`: `completions | head` — wire through the `BrokenPipe`-swallowing `emit` (it currently bypasses `emit` and could panic on a closed pipe like the other subcommands did before Phase 19). Add a test.
- [x] `tpt-cli`: has **no `tests/` and no `CHANGELOG.md`** — add an integration test harness (`assert_cmd`/`std::process::Command`) asserting exit codes, `--json` validity on success *and* failure, stdin `-`, deterministic `logfmt` ordering, and `completions bash | head` not panicking. This is what keeps the findings above fixed. (Added `tpt-cli/tests/cli.rs` with 17 integration tests; batch path mode and `--quiet` also added in the same pass.)
- [x] Remove stray tracked files still present: `TODO 1260713.md` at repo root and `history/TODO 1260713.md` (both marked removed in Phases 8/17 but still tracked).
- [x] Fix doc rot (all on publish-facing pages):
  - [x] root README "zero-allocation parsers" — jsonl allocates a `Value`/record; say "constant-memory streaming" / "amortised line buffering".
  - [x] root README geojson "line-numbered errors" — `GeoError` carries only `kind`/`path`, no line/column.
  - [x] jsonl README "AVX2… falls back at compile time" — simd-json uses runtime dispatch.
  - [x] jsonl README "parsed in place, no per-line copy" — `simd_json::from_slice` allocates per call; benchmark before publishing the claim.
  - [x] `docs/spec.txt` "3x speedups" — unsubstantiated; remove (matches README cleanup).
  - [x] mime README "no_std (with alloc)" — never allocates; drop the parenthetical.
- [x] CI: `semver` and `fuzz` jobs are `continue-on-error: true` → decorative. Make them required (or document the risk). Extend fuzz targets to the Phase 20–21 APIs (`validate`, `to_json`, `bounding_box`, `contains`, `into_typed`, writers, and the `simd` path where the overflow lives) with a `parse→to_json→parse` equality assertion.
- [x] `tpt-geo-geojson`: `GeoErrorKind`/`JsonlErrorKind` are not `#[non_exhaustive]`; add before 1.0.
- [x] `tpt-geo-geojson`: `Geometry::bounding_box` wrong across antimeridian; document or provide an antimeridian-aware variant.

## Phase 23: Innovation & usability backlog (non-blocking, from review)
- [ ] `tpt sniff` as a streaming router: detect → dispatch → one normalised NDJSON envelope per input (the natural flagship over `file`/`jq`/`mlr`).
- [ ] `tpt` `--stats` mode: counts, per-line error histogram, MB/s, field cardinality for jsonl/logfmt (turn validators into data-quality tools).
- [ ] `MimeType` confidence scores (`Confidence::{Low,Medium,High}`) from signature length + anchoring.
- [ ] Parallel JSONL parsing via `rayon` (beats the `simd` fast path; collected by construction).
- [ ] jsonl checkpoint/resume (`byte_offset()` + `resume_at`) for multi-GB ingest.
- [ ] `cron explain --verbose`: next 5 runs, DST transitions crossed, expanded field sets.
- [ ] Property-based round-trip tests (`proptest`): `write→parse==identity` (logfmt/jsonl); `parse→to_json→parse` (geojson).
- [ ] `tpt-cli`: batch/multi-path mode + `-r/--recursive`; `--quiet`; TTY color; watch mode; `-o/--output`; stdin for `logfmt` and `cron`.
