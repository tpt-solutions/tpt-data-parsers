# tpt-cron-parse

[![docs.rs](https://docs.rs/tpt-cron-parse/badge.svg)](https://docs.rs/tpt-cron-parse)
[![crates.io](https://img.shields.io/crates/v/tpt-cron-parse.svg)](https://crates.io/crates/tpt-cron-parse)

Cron expression parser with precise error reporting and human-readable output. No dependencies.

Supports standard 5-field and extended 6-field (with seconds) cron syntax.

## Features

- **5-field** (`min hour dom month dow`) and **6-field** (`sec min hour dom month dow`) cron
- **Space or tab separated** fields, as in real crontabs
- **Validated** — every value is checked against the range of its field, and trailing garbage is rejected
- **Shorthand aliases** — `@yearly`, `@annually`, `@monthly`, `@weekly`, `@daily`, `@midnight`, `@hourly`
- **Named fields** — `JAN`-`DEC` and `SUN`-`SAT`, case-insensitive
- **Precise errors** — `CronError` includes the byte position, which field failed, what was expected, and what was found
- **Human-readable** — `to_human_readable()` converts expressions to English
- **No dependencies** — pure Rust

## Usage

```rust
use tpt_cron_parse::CronExpr;

let expr = CronExpr::parse("0 9 * * 1").unwrap();
println!("{}", expr.to_human_readable()); // Every Monday at 9:00 AM

let expr = CronExpr::parse("*/5 * * * *").unwrap();
println!("{}", expr.to_human_readable()); // Every 5 minutes
```

## Shorthand aliases and named fields

```rust
use tpt_cron_parse::CronExpr;

assert_eq!(CronExpr::parse("@daily").unwrap(), CronExpr::parse("0 0 * * *").unwrap());
assert_eq!(CronExpr::parse("@weekly").unwrap(), CronExpr::parse("0 0 * * 0").unwrap());
assert_eq!(CronExpr::parse("0 9 * JAN-mar Mon-Fri").unwrap(), CronExpr::parse("0 9 * 1-3 1-5").unwrap());

// @reboot is not a periodic schedule and has no next run time.
assert!(CronExpr::parse("@reboot").is_err());
```

| Alias                   | Expansion   |
|-------------------------|-------------|
| `@yearly`, `@annually`  | `0 0 1 1 *` |
| `@monthly`              | `0 0 1 * *` |
| `@weekly`               | `0 0 * * 0` |
| `@daily`, `@midnight`   | `0 0 * * *` |
| `@hourly`               | `0 * * * *` |

## Validation

Each value is checked against the range of its field — seconds and minutes `0-59`,
hours `0-23`, day-of-month `1-31`, month `1-12`, day-of-week `0-7` (both `0` and `7`
are Sunday) — and anything left over at the end of a field is an error:

```rust
use tpt_cron_parse::CronExpr;

assert!(CronExpr::parse("0 99 * * *").is_err());        // hour out of range
assert!(CronExpr::parse("* * 32 * *").is_err());        // day-of-month out of range
assert!(CronExpr::parse("* * * * 1garbage").is_err());  // trailing input
```

## Expanding a field

`CronField::expand` returns the sorted, de-duplicated values a field matches
within the given inclusive bounds:

```rust
use tpt_cron_parse::CronExpr;

let expr = CronExpr::parse("0 1-5/2 * * *").unwrap();
assert_eq!(expr.hours.expand(0, 23), vec![1, 3, 5]);
```

## Error Reporting

```rust
use tpt_cron_parse::CronExpr;

let err = CronExpr::parse("x * * * *").unwrap_err();
println!("{}", err); // cron parse error in minutes field at position 0: expected value 0-59, found 'x'
```

## Human-Readable Examples

| Expression    | Output                          |
|---------------|---------------------------------|
| `* * * * *`   | Every minute                    |
| `0 * * * *`   | Every hour                      |
| `0 9 * * *`   | Every day at 9:00 AM            |
| `0 9 * * 1`   | Every Monday at 9:00 AM         |
| `0 9 1 * *`   | At 9:00 AM on the 1st of every month |
| `*/5 * * * *` | Every 5 minutes                 |
| `0 0 1 1 *`   | At 12:00 AM on January 1st      |

## Computing the next run time

The crate stays dependency-free by default. Enable the optional `chrono` feature
to compute the next firing time of a schedule:

```toml
[dependencies]
tpt-cron-parse = { version = "0.2", features = ["chrono"] }
```

```rust,ignore
use tpt_cron_parse::CronExpr;
use chrono::{TimeZone, Utc};

let expr = CronExpr::parse("0 9 * * 1-5").unwrap();
let after = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(); // Monday
let next = expr.next_after(after).unwrap(); // next weekday at 09:00
```

`next_after` respects the standard cron day-of-month / day-of-week OR rule and
searches at most ~4 years ahead (the maximum period of a cron schedule). For
6-field expressions it returns the very next matching second, including one
inside the current minute.

For schedules that should be evaluated in a non-UTC timezone, use the
timezone-aware `next_after_tz`, which interprets the fields as wall-clock time
in the timezone of the `after` argument (any `chrono::TimeZone`):

```rust,ignore
use tpt_cron_parse::CronExpr;
use chrono::{FixedOffset, TimeZone};

let expr = CronExpr::parse("0 9 * * *").unwrap();
let tz = FixedOffset::east_opt(8 * 3600).unwrap();
let after = tz.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
let next = expr.next_after_tz(after).unwrap(); // 09:00 in +08:00
```

## Why another cron parser?

The popular `cron` crate is excellent for scheduling, but its parse errors are
cryptic (e.g. `"expected digit at index 4"`). `tpt-cron-parse` pinpoints the exact
field, byte position, what was expected, and what was found, and can render a
schedule back as plain English — useful for CLIs, logs, and UIs.

## License

Licensed under either of [Apache License 2.0](../LICENSE-APACHE) or [MIT](../LICENSE-MIT) at your option.
