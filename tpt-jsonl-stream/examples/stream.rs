//! Example: stream-parse a JSON Lines buffer line by line.
//!
//! Run with: `cargo run -p tpt-jsonl-stream --example stream`

use std::io::BufReader;

use tpt_jsonl_stream::parse_jsonl;

fn main() {
    let data = b"{\"user\":\"alice\"}\n{\"user\":\"bob\"}\nNOT_JSON\n{\"user\":\"carol\"}\n";
    for (i, result) in parse_jsonl(BufReader::new(data.as_slice())).enumerate() {
        match result {
            Ok(value) => println!("line {}: {}", i + 1, value),
            Err(e) => eprintln!("line {}: {}", i + 1, e),
        }
    }
}
