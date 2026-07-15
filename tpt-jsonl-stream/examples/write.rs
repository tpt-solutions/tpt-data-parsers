//! Example: write records to a JSON Lines buffer.
//!
//! Run with: `cargo run -p tpt-jsonl-stream --example write`

use std::io::Cursor;

use tpt_jsonl_stream::JsonlWriter;

fn main() {
    let mut buf = Cursor::new(Vec::new());
    {
        let mut writer = JsonlWriter::new(&mut buf);
        writer.write(&serde_json::json!({"a": 1})).unwrap();
        writer.write(&serde_json::json!({"b": 2})).unwrap();
        writer.flush().unwrap();
    }
    println!("{}", String::from_utf8(buf.into_inner()).unwrap());
}
