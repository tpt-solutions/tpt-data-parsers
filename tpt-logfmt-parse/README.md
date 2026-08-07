# tpt-logfmt-parse

[![docs.rs](https://docs.rs/tpt-logfmt-parse/badge.svg)](https://docs.rs/tpt-logfmt-parse)
[![crates.io](https://img.shields.io/crates/v/tpt-logfmt-parse.svg)](https://crates.io/crates/tpt-logfmt-parse)

Zero-copy, high-performance [logfmt](https://brandur.org/logfmt) parser for Rust. No dependencies.

Logfmt is the `key=value` structured logging format used by Heroku, Datadog agents, and many Go services.

## Features

- **Zero-copy iterator** — yields `Cow<str>` key/value pairs that borrow directly from the input, allocating only when a token actually contains escapes
- **Owned convenience APIs** — `parse_to_map()` returns `HashMap<String, String>`; `parse_to_pairs()` returns `Vec<(String, String)>`, preserving order and duplicate keys
- **Multi-line streaming** — `LogfmtLinesReader<R: BufRead>` yields one parsed line at a time from Heroku router logs and friends
- **Round-tripping writer** — `write_logfmt()` / `format_pair()` quote and escape whatever the parser needs quoted and escaped
- **No dependencies** — pure Rust, no `regex`
- **Handles** quoted strings and keys, `\"` / `\\` / `\n` / `\r` / `\t` / `\xNN` escape sequences, bare keys (no `=`), and space/tab/CR/LF separators

## Usage

### Zero-copy iterator

```rust
use tpt_logfmt_parse::LogfmtParser;

let input = r#"level=info msg="hello world" latency=42ms"#;
for pair in LogfmtParser::new(input) {
    let (key, value) = pair.unwrap();
    println!("{key} = {value}");
}
```

### Owned HashMap

```rust
use tpt_logfmt_parse::parse_to_map;

let map = parse_to_map(r#"level=error msg="disk full" retries=3"#).unwrap();
println!("{}", map["msg"]); // disk full
```

### Owned pairs (order and duplicates preserved)

```rust
use tpt_logfmt_parse::parse_to_pairs;

let pairs = parse_to_pairs("tag=a tag=b level=info").unwrap();
assert_eq!(pairs[0], ("tag".to_string(), "a".to_string()));
assert_eq!(pairs[1], ("tag".to_string(), "b".to_string()));
```

### Multi-line sources

```rust
use std::io::BufReader;
use tpt_logfmt_parse::LogfmtLinesReader;

let data = b"at=info status=200\nat=warn status=503\n";
for line in LogfmtLinesReader::new(BufReader::new(data.as_slice())) {
    let pairs = line.unwrap();
    println!("{} fields", pairs.len());
}
```

### Writing

```rust
use tpt_logfmt_parse::{parse_to_pairs, write_logfmt};

let pairs = vec![("msg".to_string(), "a \"quoted\"\nline".to_string())];
let line = write_logfmt(pairs.clone());
assert_eq!(line, r#"msg="a \"quoted\"\nline""#);
assert_eq!(parse_to_pairs(&line).unwrap(), pairs);
```

## Why another logfmt parser?

Most Rust logfmt handling either reaches for a full regex engine or a
general-purpose key/value splitter that allocates per pair. `tpt-logfmt-parse` is
hand-rolled (no `regex`), **zero-copy in the iterator path**, and handles the
tricky cases — quoted strings, `\"` / `\\` escapes, bare keys — that naive
split-on-space approaches get wrong. Parsing and writing share one escape
implementation, so `write_logfmt` output always parses back to what went in.

## License

Licensed under either of [Apache License 2.0](../LICENSE-APACHE) or [MIT](../LICENSE-MIT) at your option.
