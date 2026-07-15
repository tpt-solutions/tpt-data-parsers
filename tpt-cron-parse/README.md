# tpt-cron-parse

[![docs.rs](https://docs.rs/tpt-cron-parse/badge.svg)](https://docs.rs/tpt-cron-parse)
[![crates.io](https://img.shields.io/crates/v/tpt-cron-parse.svg)](https://crates.io/crates/tpt-cron-parse)

Cron expression parser with precise error reporting and human-readable output. No dependencies.

Supports standard 5-field and extended 6-field (with seconds) cron syntax.

## Features

- **5-field** (`min hour dom month dow`) and **6-field** (`sec min hour dom month dow`) cron
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

## Error Reporting

```rust
use tpt_cron_parse::CronExpr;

let err = CronExpr::parse("x * * * *").unwrap_err();
println!("{}", err); // cron parse error in minutes field at position 0: expected digit, found 'x'
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
tpt-cron-parse = { version = "0.1", features = ["chrono"] }
```

```rust,ignore
use tpt_cron_parse::CronExpr;
use chrono::{TimeZone, Utc};

let expr = CronExpr::parse("0 9 * * 1-5").unwrap();
let after = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(); // Monday
let next = expr.next_after(after).unwrap(); // next weekday at 09:00
```

`next_after` respects the standard cron day-of-month / day-of-week OR rule and
searches at most ~4 years ahead (the maximum period of a cron schedule).

## License

Licensed under either of [Apache License 2.0](../LICENSE-APACHE) or [MIT](../LICENSE-MIT) at your option.
