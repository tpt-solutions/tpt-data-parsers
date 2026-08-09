# Changelog

All notable changes to this crate are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Shorthand aliases `@yearly`, `@annually`, `@monthly`, `@weekly`, `@daily`,
  `@midnight` and `@hourly`, expanded before field parsing. `@reboot` is rejected
  with a `CronError` because it is not a periodic schedule and has no next run time.
- Case-insensitive named months (`JAN`-`DEC`) and days-of-week (`SUN`-`SAT`) in
  values, ranges, lists and steps, alongside the numeric forms.
- `CronField::expand`, returning the sorted, de-duplicated set of values a field
  matches within inclusive bounds.
- `CronExpr::next_after_tz` (under the `chrono` feature), a timezone-aware variant
  of `next_after` that interprets the schedule as wall-clock time in any
  `chrono::TimeZone`.

### Fixed

- Field values are now validated against the range of their field (seconds and
  minutes 0-59, hours 0-23, day-of-month 1-31, month 1-12, day-of-week 0-7 with
  both 0 and 7 meaning Sunday). Expressions such as `0 99 * * *` are rejected
  instead of parsing into nonsense.
- Trailing input after a field is rejected, so `* * * * 1garbage` and
  `* * * * *DROP TABLE` now produce a `CronError` instead of parsing.
- A step over a range no longer ignores the range end: `1-5/2` expands to
  `[1, 3, 5]` instead of continuing to the maximum of the field.
- 6-field `next_after` no longer skips the current minute: `30 0 9 * * *` after
  `09:00:00` returns `09:00:30` the same day, and `*/1 * * * * *` after
  `09:00:00` returns `09:00:01`.
- Fields may be separated by tabs as well as spaces; splitting and field counting
  now use the same rule everywhere.

## [0.2.0] - 2026-08-08

### Added

- `CronExpr::upcoming` (behind the optional `chrono` feature), returning an `Upcoming`
  iterator over successive firing times strictly after a starting point.

## [0.1.0] - 2026-07-17

Initial release.

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
