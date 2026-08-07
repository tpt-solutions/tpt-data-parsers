# tpt-cli

Command-line front-end for the [`tpt-data-parsers`](../README.md) workspace. It wraps
the five parser crates behind a single `tpt` binary with one subcommand per format.

This crate is **not published to crates.io** — it is a convenience tool. It is listed
in the workspace but deliberately left out of `publish.yml`.

## Install

```bash
cargo install --path tpt-cli
```

## Global output flags

Every subcommand accepts output-control flags:

- `--json` — emit machine-readable JSON instead of human text.
- `--format <human|json>` — equivalent to `--json` when set to `json`.

For example:

```bash
tpt cron "*/5 * * * *" --next --json
tpt logfmt 'level=info msg="hello world"' --json
tpt jsonl data.jsonl --format json
```

## Usage

```bash
# Describe a cron expression (add --next for the next firing time)
tpt cron "0 9 * * 1-5"
tpt cron "*/5 * * * *" --next

# Cron --next honours a timezone. Default is UTC.
# Use UTC, local, or a fixed offset like +08:00 / -05:00.
tpt cron "0 9 * * *" --next --timezone "+08:00"
tpt cron "0 9 * * *" --next -z local

# Detect a file's type from its magic bytes (falls back to extension)
tpt mime some-file.bin

# Validate a GeoJSON document (streamed; only 8 KB read for mime detection)
tpt geojson map.geojson

# Parse a logfmt line (decoded values, no literal escapes)
tpt logfmt 'level=info msg="hello world"'

# Count records in a JSON Lines file
tpt jsonl data.jsonl
```

### Reading from stdin

`mime`, `geojson`, and `jsonl` accept `-` as the path to read from standard input:

```bash
cat map.geojson | tpt geojson -
echo '{"x":1}' | tpt jsonl -
head -c 8192 some-file.bin | tpt mime -
```

## Sniff

`tpt sniff <path>` detects a file's type and auto-dispatches to the matching parser.
It first runs MIME detection (binary formats), then falls back to content sniffing to
classify JSON, GeoJSON, JSON Lines, and logfmt. With `--json` it emits the detected
format plus the parsed/validated data.

```bash
tpt sniff exports.geojson
tpt sniff events.jsonl --json
cat events.logfmt | tpt sniff -
```

## Shell completions

Generate shell completion scripts with the `completions` subcommand and your shell:

```bash
tpt completions bash > /etc/bash_completion.d/tpt
tpt completions zsh  > ~/.zfunc/_tpt
tpt completions fish > ~/.config/fish/completions/tpt.fish
tpt completions powershell > tpt.ps1
```

`clap_complete` is the only extra dependency added for this feature.

## License

Licensed under either of [Apache License 2.0](../LICENSE-APACHE) or [MIT](../LICENSE-MIT)
at your option.
