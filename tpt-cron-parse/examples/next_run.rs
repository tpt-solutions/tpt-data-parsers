//! Example: compute the next time a cron schedule fires (requires the `chrono` feature).
//!
//! Run with: `cargo run -p tpt-cron-parse --example next_run --features chrono`

use chrono::{TimeZone, Utc};

use tpt_cron_parse::CronExpr;

fn main() {
    let expr = CronExpr::parse("0 9 * * 1-5").unwrap();
    let after = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(); // a Monday
    let next = expr.next_after(after).unwrap();
    println!("next run: {}", next);
}
