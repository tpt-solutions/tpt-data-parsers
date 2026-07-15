//! Example: parse a crontab-style expression and describe it.
//!
//! Run with: `cargo run -p tpt-cron-parse --example crontab`

use tpt_cron_parse::CronExpr;

fn main() {
    let expr = CronExpr::parse("0 9 * * 1-5").expect("valid cron expression");
    println!("Parsed as 6-field cron: {}", expr.is_6_field());
    println!("Human readable: {}", expr.to_human_readable());
}
