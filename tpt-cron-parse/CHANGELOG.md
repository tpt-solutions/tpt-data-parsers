# Changelog

All notable changes to this crate are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-08-08

Initial published release.

### Added

- `CronExpr::parse` for standard 5-field and 6-field (with seconds) cron expressions.
- `CronExpr::to_human_readable` translating a schedule into English.
- `CronError` with the exact byte position, field, expected value, and found character.
- `CronExpr::next_after` (behind the optional `chrono` feature) computing the next
  firing time, honouring the cron day-of-month / day-of-week OR rule.

### Fixed

- Descending ranges such as `5-1` are now rejected with a `CronError` instead of
  producing a silently empty set.
- Zero step values such as `*/0` are now rejected with a `CronError`.
