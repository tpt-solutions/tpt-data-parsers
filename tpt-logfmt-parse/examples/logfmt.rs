//! Example: parse a single logfmt line into an owned map.
//!
//! Run with: `cargo run -p tpt-logfmt-parse --example logfmt`

use tpt_logfmt_parse::parse_to_map;

fn main() {
    let line = r#"level=info msg="hello world" retries=3"#;
    let map = parse_to_map(line).expect("valid logfmt");
    println!("level   = {}", map["level"]);
    println!("msg     = {}", map["msg"]);
    println!("retries = {}", map["retries"]);
}
