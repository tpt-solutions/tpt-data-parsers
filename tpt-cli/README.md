# tpt-cli

Command-line front-end for the [`tpt-data-parsers`](../README.md) workspace. It wraps
the five parser crates behind a single `tpt` binary with one subcommand per format.

This crate is **not published to crates.io** — it is a convenience tool. It is listed
in the workspace but deliberately left out of `publish.yml`.

## Install

```bash
cargo install --path tpt-cli
```

## Usage

```bash
# Describe a cron expression (add --next for the next firing time)
tpt cron "0 9 * * 1-5"
tpt cron "*/5 * * * *" --next

# Detect a file's type from its magic bytes (falls back to extension)
tpt mime some-file.bin

# Validate a GeoJSON file
tpt geojson map.geojson

# Parse a logfmt line
tpt logfmt 'level=info msg="hello world"'

# Count records in a JSON Lines file
tpt jsonl data.jsonl
```

## License

Licensed under either of [Apache License 2.0](../LICENSE-APACHE) or [MIT](../LICENSE-MIT)
at your option.
