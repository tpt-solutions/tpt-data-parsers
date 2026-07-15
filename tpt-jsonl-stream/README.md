# tpt-jsonl-stream

[![docs.rs](https://docs.rs/tpt-jsonl-stream/badge.svg)](https://docs.rs/tpt-jsonl-stream)
[![crates.io](https://img.shields.io/crates/v/tpt-jsonl-stream.svg)](https://crates.io/crates/tpt-jsonl-stream)

Streaming, zero-allocation JSON Lines (`.jsonl`) parser for Rust.

AI and data-engineering pipelines use massive JSONL files. Standard parsers load the whole file into RAM. This crate streams line-by-line from any [`BufRead`](https://doc.rust-lang.org/std/io/trait.BufRead.html).

## Features

- **Streaming** — reads one line at a time; constant memory regardless of file size
- **Error context** — every error carries the exact 1-based line number
- **Optional SIMD** — enable the `simd` feature for 3× JSON parse throughput on AVX2 CPUs
- **Simple iterator API** — `for value in parse_jsonl(reader) { ... }`

## Usage

```rust,no_run
use tpt_jsonl_stream::parse_jsonl;
use std::io::BufReader;
use std::fs::File;

let f = File::open("data.jsonl").unwrap();
for result in parse_jsonl(BufReader::new(f)) {
    let value = result.unwrap();
    println!("{}", value["name"]);
}
```

### SIMD acceleration

```toml
[dependencies]
tpt-jsonl-stream = { version = "0.1", features = ["simd"] }
```

Requires an AVX2-capable CPU. Falls back to standard parsing on unsupported hardware at compile time.

## Error handling

```rust
use tpt_jsonl_stream::parse_jsonl;
use std::io::BufReader;

let data = b"{\"a\":1}\nNOT_JSON\n{\"c\":3}\n";
for result in parse_jsonl(BufReader::new(data.as_slice())) {
    match result {
        Ok(v) => println!("ok: {}", v),
        Err(e) => eprintln!("line {}: {}", e.line, e.kind),
    }
}
```

## Writing

The crate is also a JSON Lines *writer*. [`JsonlWriter`] emits one
newline-terminated JSON value per call; [`write_jsonl`] writes a whole
sequence in one go.

```rust
use tpt_jsonl_stream::JsonlWriter;
use std::io::Cursor;

let mut buf = Cursor::new(Vec::new());
{
    let mut writer = JsonlWriter::new(&mut buf);
    writer.write(&serde_json::json!({"a": 1})).unwrap();
    writer.write(&serde_json::json!({"b": 2})).unwrap();
}
assert_eq!(String::from_utf8(buf.into_inner()).unwrap(), "{\"a\":1}\n{\"b\":2}\n");
```

## License

Licensed under either of [Apache License 2.0](../LICENSE-APACHE) or [MIT](../LICENSE-MIT) at your option.
