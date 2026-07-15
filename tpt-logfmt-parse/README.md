# tpt-logfmt-parse

[![docs.rs](https://docs.rs/tpt-logfmt-parse/badge.svg)](https://docs.rs/tpt-logfmt-parse)
[![crates.io](https://img.shields.io/crates/v/tpt-logfmt-parse.svg)](https://crates.io/crates/tpt-logfmt-parse)

Zero-copy, high-performance [logfmt](https://brandur.org/logfmt) parser for Rust. No dependencies.

Logfmt is the `key=value` structured logging format used by Heroku, Datadog agents, and many Go services.

## Features

- **Zero-copy iterator** — yields `(&str, &str)` slices directly into the input string
- **Owned convenience API** — `parse_to_map()` returns `HashMap<String, String>` with escape sequences resolved
- **No dependencies** — pure Rust, no `regex`, no allocations in the iterator path
- **Handles** quoted strings, `\"` / `\\` / `\n` / `\t` escape sequences, and bare keys (no `=`)

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

## License

Licensed under either of [Apache License 2.0](../LICENSE-APACHE) or [MIT](../LICENSE-MIT) at your option.
