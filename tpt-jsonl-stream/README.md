# tpt-jsonl-stream

[![docs.rs](https://docs.rs/tpt-jsonl-stream/badge.svg)](https://docs.rs/tpt-jsonl-stream)
[![crates.io](https://img.shields.io/crates/v/tpt-jsonl-stream.svg)](https://crates.io/crates/tpt-jsonl-stream)

Streaming, constant-memory JSON Lines (`.jsonl`) parser for Rust.

AI and data-engineering pipelines use massive JSONL files. Standard parsers load the whole file into RAM. This crate streams line-by-line from any [`BufRead`](https://doc.rust-lang.org/std/io/trait.BufRead.html).

## Features

- **Streaming** — reads one line at a time; constant memory regardless of file size
- **Bounded memory** — a configurable max line length (16 MiB by default) means a
  newline-free stream can never exhaust RAM
- **Error context** — every error carries the exact 1-based line number
- **Typed records** — `into_typed::<T>()` deserializes each line into your own struct
- **Optional SIMD** — enable the `simd` feature to use `simd_json` for parsing; it
  parses in place without an extra copy (benchmark on your own data before relying on it)
- **Async streaming** — enable the `tokio` feature for an `AsyncJsonlReader` over
  `tokio`'s `AsyncBufRead`
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

### Typed records

[`JsonlReader::into_typed`] deserializes every line into a type of your choice and
still reports the line number for both syntax errors and type mismatches.

```rust
use serde::Deserialize;
use std::io::BufReader;
use tpt_jsonl_stream::parse_jsonl;

#[derive(Deserialize)]
struct User {
    name: String,
    age: u8,
}

let data = b"{\"name\":\"alice\",\"age\":30}\n{\"name\":\"bob\",\"age\":41}\n";
let users: Vec<User> = parse_jsonl(BufReader::new(data.as_slice()))
    .into_typed::<User>()
    .collect::<Result<_, _>>()
    .unwrap();
assert_eq!(users[0].name, "alice");
assert_eq!(users[1].age, 41);
```

### Line length limits

Lines are buffered up to [`DEFAULT_MAX_LINE_LENGTH`] (16 MiB) by default. An
over-long line is discarded rather than buffered, reported as
[`JsonlErrorKind::LineTooLong`], and the stream resumes on the next line.

```rust
use std::io::BufReader;
use tpt_jsonl_stream::{parse_jsonl, JsonlErrorKind};

let data = b"{\"a\":1}\n[0,1,2,3,4,5,6,7,8,9]\n{\"b\":2}\n";
let mut reader = parse_jsonl(BufReader::new(data.as_slice())).with_max_line_length(8);
assert_eq!(reader.next().unwrap().unwrap()["a"], 1);
let err = reader.next().unwrap().unwrap_err();
assert!(matches!(err.kind, JsonlErrorKind::LineTooLong { limit: 8 }));
assert_eq!(reader.next().unwrap().unwrap()["b"], 2);
```

Pass `usize::MAX` for unbounded buffering.

### SIMD acceleration

```toml
[dependencies]
tpt-jsonl-stream = { version = "0.2", features = ["simd"] }
```

No special CPU target is required at compile time. `simd_json` selects the best SIMD
implementation available at runtime, falling back to a portable parser automatically on
hardware without AVX2.
Lines are parsed into `simd_json`'s own DOM, which does allocate; benchmark on your
own data before relying on it for a speedup.

## Blank lines

Only lines that are empty or consist solely of ASCII whitespace (space, tab, carriage
return, newline) are skipped. Any other content — including a line holding just a
non-breaking space — is handed to the parser, and error offsets refer to the original
line because leading whitespace is never stripped.

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
sequence in one go. Each record is serialized into an internal buffer and emitted
as a single write, so a value that fails to serialize leaves no partial line behind.

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

## Why another JSONL crate?

Rolling your own `lines().map(from_str)` works, but drops line numbers and
allocates a buffer per line; `serde_json::Deserializer::from_reader` handles one
concatenated stream but not mixed or blank-line `.jsonl`. `tpt-jsonl-stream`
streams one value per line with precise line-numbered errors, a symmetric writer,
and an optional SIMD fast path.

## License

Licensed under either of [Apache License 2.0](../LICENSE-APACHE) or [MIT](../LICENSE-MIT) at your option.
