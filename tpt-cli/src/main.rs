use chrono::{DateTime, FixedOffset, Local, TimeZone, Utc};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use std::io::{BufRead, BufReader, IsTerminal, Read, Seek, Write};
use std::path::Path;
use std::process::ExitCode;
use std::str::FromStr;
use std::time::Instant;

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    Human,
    Json,
}

#[derive(Parser)]
#[command(name = "tpt", version, about = "TPT data parser CLI")]
struct Cli {
    /// Emit machine-readable JSON instead of human text
    #[arg(long, global = true)]
    json: bool,
    /// Output format: human text or json
    #[arg(long, value_enum, global = true, default_value = "human")]
    format: OutputFormat,
    /// Suppress human-readable output (still emits JSON and errors)
    #[arg(long, global = true)]
    quiet: bool,
    /// Write normal output to a file instead of stdout (errors still go to stderr)
    #[arg(long, short, global = true)]
    output: Option<String>,
    /// Print summary statistics instead of per-record output (jsonl/logfmt)
    #[arg(long, global = true)]
    stats: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Parse and describe a cron expression
    Cron {
        /// Cron expression (5 or 6 fields). Use `-` to read it from stdin.
        expr: String,
        /// Also compute the next run time (needs the chrono feature)
        #[arg(long)]
        next: bool,
        /// Timezone for --next/--explain: UTC, local, or a fixed offset like +08:00.
        /// Affects when the schedule fires, not just how it is displayed.
        #[arg(long, short = 'z', default_value = "UTC")]
        timezone: String,
        /// Print a verbose explanation: expanded field sets, the next 5 runs, and
        /// any DST transitions crossed between them.
        #[arg(long)]
        explain: bool,
    },
    /// Detect a file's MIME type from its bytes (falls back to extension)
    Mime {
        /// One or more paths, or - for stdin
        #[arg(required = true)]
        paths: Vec<String>,
        /// Recurse into directories, expanding them to their contained files
        #[arg(short, long)]
        recursive: bool,
    },
    /// Validate a GeoJSON document
    Geojson {
        /// One or more paths, or - for stdin
        #[arg(required = true)]
        paths: Vec<String>,
        /// Recurse into directories, expanding them to their contained files
        #[arg(short, long)]
        recursive: bool,
    },
    /// Parse one or more logfmt lines
    Logfmt {
        /// One or more logfmt lines, or - for stdin
        #[arg(required = true)]
        lines: Vec<String>,
    },
    /// Count the records in a JSON Lines file
    Jsonl {
        /// One or more paths, or - for stdin
        #[arg(required = true)]
        paths: Vec<String>,
        /// Recurse into directories, expanding them to their contained files
        #[arg(short, long)]
        recursive: bool,
        /// Follow the file and print new records as they are appended (tail -f)
        #[arg(long)]
        watch: bool,
    },
    /// Detect a path's type and auto-dispatch to the matching parser
    Sniff {
        /// One or more paths, or - for stdin
        #[arg(required = true)]
        paths: Vec<String>,
        /// Recurse into directories, expanding them to their contained files
        #[arg(short, long)]
        recursive: bool,
        /// Emit one normalised NDJSON envelope per input (streaming router) instead
        /// of the default pretty/JSON classification summary.
        #[arg(long)]
        router: bool,
    },
    /// Generate shell completion scripts
    Completions {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

/// A byte sink for normal (non-error) output. Writes to stdout by default, or to
/// a file when `--output` is given. Broken-pipe errors are swallowed so the CLI
/// never panics when piped into `head`/`less`.
struct Output {
    sink: Box<dyn Write>,
    color: bool,
}

impl Output {
    fn stdout() -> Self {
        let color = std::io::stdout().is_terminal();
        Output {
            sink: Box::new(std::io::stdout()),
            color,
        }
    }

    fn file(path: &str) -> Result<Self, String> {
        let f = std::fs::File::create(path)
            .map_err(|e| format!("cannot open output file {path}: {e}"))?;
        Ok(Output {
            sink: Box::new(f),
            color: false,
        })
    }

    fn writeln(&mut self, s: &str) -> Result<(), String> {
        self.write_all(s.as_bytes())?;
        self.write_all(b"\n")
    }

    fn write_all(&mut self, b: &[u8]) -> Result<(), String> {
        match self.sink.write_all(b) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    fn flush(&mut self) -> Result<(), String> {
        self.sink.flush().map_err(|e| e.to_string())
    }

    /// Wrap `text` in an ANSI escape if color output is enabled.
    fn paint(&self, text: &str, code: &str) -> String {
        if self.color {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let format = if cli.json {
        OutputFormat::Json
    } else {
        cli.format
    };
    let mut out = match &cli.output {
        Some(path) => match Output::file(path) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
        },
        None => Output::stdout(),
    };
    match run(&cli, format, &mut out) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            emit_err(format, &e);
            ExitCode::FAILURE
        }
    }
}

/// Write bytes to stdout, swallowing a closed-pipe error so the CLI never
/// panics when piped into `head`/`less` (the completions subcommand writes
/// through here too).
fn emit_bytes(buf: &[u8]) -> Result<(), String> {
    let mut out = std::io::stdout().lock();
    match out.write_all(buf) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// Emit `value` as JSON when in JSON mode, or a plain `human` representation
/// otherwise. The `human` closure is only called in human mode.
fn emit_json_or(
    out: &mut Output,
    format: OutputFormat,
    value: serde_json::Value,
    human: impl FnOnce() -> String,
    quiet: bool,
) -> Result<(), String> {
    match format {
        OutputFormat::Json => {
            let s = serde_json::to_string(&value).map_err(|e| e.to_string())?;
            out.writeln(&s)
        }
        OutputFormat::Human => {
            if quiet {
                Ok(())
            } else {
                out.writeln(&human())
            }
        }
    }
}

fn emit_err(format: OutputFormat, msg: &str) {
    match format {
        OutputFormat::Json => {
            let mut obj = serde_json::Map::new();
            obj.insert("ok".into(), serde_json::Value::Bool(false));
            obj.insert("error".into(), serde_json::Value::String(msg.into()));
            let s = serde_json::to_string(&serde_json::Value::Object(obj)).unwrap();
            eprintln!("{s}");
        }
        OutputFormat::Human => {
            eprintln!("error: {msg}");
        }
    }
}

fn open_read(path: &str) -> Result<Box<dyn Read>, String> {
    if path == "-" {
        Ok(Box::new(std::io::stdin()))
    } else {
        Ok(Box::new(
            std::fs::File::open(path).map_err(|e| e.to_string())?,
        ))
    }
}

#[derive(Clone)]
enum TzSpec {
    Utc,
    Local,
    Offset(FixedOffset),
}

fn parse_tz(spec: &str) -> Result<TzSpec, String> {
    match spec.to_ascii_lowercase().as_str() {
        "utc" | "z" | "gmt" => Ok(TzSpec::Utc),
        "local" => Ok(TzSpec::Local),
        other => FixedOffset::from_str(other)
            .map(TzSpec::Offset)
            .map_err(|_| {
                format!("invalid timezone: {spec} (use UTC, local, or a fixed offset like +08:00)")
            }),
    }
}

impl TzSpec {
    /// The current instant in this timezone, as a concrete `DateTime` whose
    /// timezone type matches whichever arm we are in. `next_after_tz` is
    /// generic over the timezone, so each arm supplies its own concrete type.
    fn now(&self) -> Now {
        match self {
            TzSpec::Utc => Now::Utc(Utc::now()),
            TzSpec::Local => Now::Local(Local::now()),
            TzSpec::Offset(f) => Now::Offset(Utc::now().with_timezone(f)),
        }
    }
}

/// A `DateTime` in one of the concrete timezone types this CLI supports.
enum Now {
    Utc(DateTime<Utc>),
    Local(DateTime<Local>),
    Offset(DateTime<FixedOffset>),
}

fn run(cli: &Cli, format: OutputFormat, out: &mut Output) -> Result<(), String> {
    let quiet = cli.quiet;
    match &cli.command {
        Command::Cron {
            expr,
            next,
            timezone,
            explain,
        } => {
            let expr = if expr == "-" {
                read_stdin_line()?
            } else {
                expr.clone()
            };
            let parsed = tpt_cron_parse::CronExpr::parse(&expr).map_err(|e| e.to_string())?;
            let human = parsed.to_human_readable();
            if *explain {
                return explain_output(&parsed, &expr, timezone, format, out, quiet);
            }
            let next_run = if *next {
                let tz = parse_tz(timezone)?;
                Some(next_run_string(&parsed, &tz)?)
            } else {
                None
            };
            if format == OutputFormat::Json {
                let mut obj = serde_json::Map::new();
                obj.insert("ok".into(), serde_json::Value::Bool(true));
                obj.insert("expression".into(), serde_json::Value::String(expr));
                obj.insert("human".into(), serde_json::Value::String(human.clone()));
                if let Some(n) = next_run {
                    obj.insert("next".into(), serde_json::Value::String(n));
                }
                emit_json_or(
                    out,
                    format,
                    serde_json::Value::Object(obj),
                    || human.clone(),
                    quiet,
                )
            } else {
                emit_json_or(
                    out,
                    format,
                    {
                        let mut o = serde_json::Map::new();
                        o.insert("ok".into(), serde_json::Value::Bool(true));
                        o.insert("expression".into(), serde_json::Value::String(expr));
                        o.insert("human".into(), serde_json::Value::String(human.clone()));
                        if let Some(n) = &next_run {
                            o.insert("next".into(), serde_json::Value::String(n.clone()));
                        }
                        serde_json::Value::Object(o)
                    },
                    || match &next_run {
                        Some(n) => format!("{human}\nnext run: {n}"),
                        None => human.clone(),
                    },
                    quiet,
                )
            }
        }
        Command::Mime { paths, recursive } => {
            let expanded = expand_paths(paths, *recursive)?;
            let mut last = Ok(());
            for path in &expanded {
                let mime = detect_mime(path)?;
                last = match mime {
                    Some(m) => {
                        let obj = || {
                            let mut o = serde_json::Map::new();
                            o.insert("ok".into(), serde_json::Value::Bool(true));
                            o.insert("path".into(), serde_json::Value::String(path.clone()));
                            o.insert("mime".into(), serde_json::Value::String(m.as_str().into()));
                            o.insert(
                                "extension".into(),
                                serde_json::Value::String(m.extension().into()),
                            );
                            serde_json::Value::Object(o)
                        };
                        let human = || m.as_str().to_string();
                        emit_json_or(out, format, obj(), || human().clone(), quiet)
                    }
                    None => Err("unknown type".into()),
                };
            }
            last
        }
        Command::Geojson { paths, recursive } => {
            let expanded = expand_paths(paths, *recursive)?;
            let mut last = Ok(());
            for path in &expanded {
                let reader = BufReader::new(open_read(path)?);
                let geo = tpt_geo_geojson::parse_reader(reader).map_err(|e| e.to_string())?;
                let data = serde_json::to_value(&geo).map_err(|e| e.to_string())?;
                let obj = {
                    let mut o = serde_json::Map::new();
                    o.insert("ok".into(), serde_json::Value::Bool(true));
                    o.insert("path".into(), serde_json::Value::String(path.clone()));
                    o.insert("valid".into(), serde_json::Value::Bool(true));
                    o.insert("data".into(), data);
                    serde_json::Value::Object(o)
                };
                let human = "valid".to_string();
                last = emit_json_or(out, format, obj, || human.clone(), quiet);
            }
            last
        }
        Command::Logfmt { lines } => {
            if cli.stats {
                return logfmt_stats(lines, format, out, quiet);
            }
            // Expand "-" (stdin) into the actual lines so `-` can be batched.
            let mut expanded: Vec<String> = Vec::new();
            for l in lines {
                if l == "-" {
                    let stdin = std::io::stdin();
                    for line in stdin.lock().lines() {
                        expanded.push(line.map_err(|e| e.to_string())?);
                    }
                } else {
                    expanded.push(l.clone());
                }
            }
            let mut last = Ok(());
            for line in &expanded {
                // `parse_to_pairs` preserves key order and duplicate keys,
                // unlike `parse_to_map` (a HashMap that drops both).
                let pairs = tpt_logfmt_parse::parse_to_pairs(line).map_err(|e| e.to_string())?;
                let obj = || {
                    let mut o = serde_json::Map::new();
                    o.insert("ok".into(), serde_json::Value::Bool(true));
                    let fields: Vec<serde_json::Value> = pairs
                        .iter()
                        .map(|(k, v)| {
                            let mut m = serde_json::Map::new();
                            m.insert("key".into(), serde_json::Value::String(k.clone()));
                            m.insert("value".into(), serde_json::Value::String(v.clone()));
                            serde_json::Value::Object(m)
                        })
                        .collect();
                    o.insert("fields".into(), serde_json::Value::Array(fields));
                    serde_json::Value::Object(o)
                };
                let human = || {
                    pairs
                        .iter()
                        .map(|(k, v)| format!("{k} = {v}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                };
                last = emit_json_or(out, format, obj(), || human().clone(), quiet);
            }
            last
        }
        Command::Jsonl {
            paths,
            recursive,
            watch,
        } => {
            if *watch {
                return watch_jsonl(paths, *recursive, format, out, quiet);
            }
            if cli.stats {
                return jsonl_stats(paths, *recursive, format, out, quiet);
            }
            let expanded = expand_paths(paths, *recursive)?;
            let mut last = Ok(());
            for path in &expanded {
                let reader = BufReader::new(open_read(path)?);
                let mut count = 0u64;
                for res in tpt_jsonl_stream::parse_jsonl(reader) {
                    res.map_err(|e| e.to_string())?;
                    count += 1;
                }
                let obj = || {
                    let mut o = serde_json::Map::new();
                    o.insert("ok".into(), serde_json::Value::Bool(true));
                    o.insert("path".into(), serde_json::Value::String(path.clone()));
                    o.insert("records".into(), serde_json::Value::Number(count.into()));
                    serde_json::Value::Object(o)
                };
                let human = || format!("records: {count}");
                last = emit_json_or(out, format, obj(), || human().clone(), quiet);
            }
            last
        }
        Command::Sniff {
            paths,
            recursive,
            router,
        } => {
            let expanded = expand_paths(paths, *recursive)?;
            if *router {
                // Streaming router: one normalised NDJSON envelope per input.
                let mut any_unknown = false;
                for path in &expanded {
                    let envelope = router_envelope(path)?;
                    if envelope_unknown(&envelope) {
                        any_unknown = true;
                    }
                    out.writeln(&envelope)?;
                }
                out.flush()?;
                if any_unknown {
                    return Err("unknown type".into());
                }
                return Ok(());
            }
            // A single JSON envelope combining every inspected path, so the
            // output is stable in both human and JSON modes regardless of input.
            // A path that classifies as `unknown` is still a valid result, but
            // mirrors `mime` (which exits 1 when nothing is detected) by making
            // the process exit non-zero so CI/pipelines can treat it as a failure.
            let mut results: Vec<serde_json::Value> = Vec::new();
            let mut any_unknown = false;
            for path in &expanded {
                let (category, truncated) = classify_path(path)?;
                if matches!(category, SniffCategory::Unknown) {
                    any_unknown = true;
                }
                results.push(category_to_json(path, category, truncated, format, quiet, out)?);
            }
            if format == OutputFormat::Json {
                let mut arr = serde_json::Map::new();
                arr.insert("ok".into(), serde_json::Value::Bool(!any_unknown));
                arr.insert("results".into(), serde_json::Value::Array(results));
                out.writeln(
                    &serde_json::to_string(&serde_json::Value::Object(arr))
                        .map_err(|e| e.to_string())?,
                )?;
            }
            // In human mode each path was emitted per-path inside category_to_json.
            if any_unknown {
                return Err("unknown type".into());
            }
            Ok(())
        }
        Command::Completions { shell } => {
            let mut cmd = Cli::command();
            let mut buf = Vec::new();
            clap_complete::generate(*shell, &mut cmd, "tpt", &mut buf);
            emit_bytes(&buf)
        }
    }
}

/// Compute the next run time as a display string, using the timezone-aware
/// `next_after_tz` so the timezone controls *when* the schedule fires.
fn next_run_string(parsed: &tpt_cron_parse::CronExpr, tz: &TzSpec) -> Result<String, String> {
    match tz.now() {
        Now::Utc(t) => parsed
            .next_after_tz(t)
            .map(|t| t.to_rfc3339())
            .ok_or_else(|| "none within 8 years".to_string()),
        Now::Local(t) => parsed
            .next_after_tz(t)
            .map(|t| t.to_rfc3339())
            .ok_or_else(|| "none within 8 years".to_string()),
        Now::Offset(t) => parsed
            .next_after_tz(t)
            .map(|t| t.to_string())
            .ok_or_else(|| "none within 8 years".to_string()),
    }
}

fn detect_mime(path: &str) -> Result<Option<tpt_mime_pure::MimeType>, String> {
    if path == "-" {
        let mut buf = [0u8; 8192];
        let mut stdin = std::io::stdin();
        let mut filled = 0;
        while filled < buf.len() {
            match stdin.read(&mut buf[filled..]) {
                Ok(0) => break,
                Ok(n) => filled += n,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e.to_string()),
            }
        }
        Ok(tpt_mime_pure::detect(&buf[..filled]))
    } else {
        let mime = tpt_mime_pure::detect_file(path).map_err(|e| e.to_string())?;
        if mime.is_some() {
            return Ok(mime);
        }
        // Use `Path::extension` so a file with no extension (e.g. `exe`) is not
        // mistaken for a `.exe` by an over-eager `rsplit('.')`.
        Ok(Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .and_then(tpt_mime_pure::detect_by_extension))
    }
}

enum SniffCategory {
    Binary(tpt_mime_pure::MimeType),
    Json(serde_json::Value),
    GeoJson(tpt_geo_geojson::GeoJson),
    Jsonl(usize),
    Logfmt(Vec<(String, String)>),
    Unknown,
}

/// How many bytes `sniff` reads from the head of a file before classifying.
///
/// Detection (magic bytes, a JSON/XML head, the first jsonl/logfmt line) almost
/// always succeeds within this window; the remainder of a large file is not
/// needed. When the file is larger than this, `truncated: true` is reported so
/// callers know the classification is based on a prefix.
const SNIFF_READ_LIMIT: usize = 8 * 1024 * 1024;

fn classify_path(path: &str) -> Result<(SniffCategory, bool), String> {
    let mut raw = open_read(path)?;
    let mut buf = Vec::new();
    let limit = SNIFF_READ_LIMIT as u64;
    {
        let mut reader = (&mut *raw).take(limit);
        reader.read_to_end(&mut buf).map_err(|e| e.to_string())?;
    }
    // Probe for any byte beyond the window to know whether we truncated.
    let mut probe = [0u8; 1];
    let truncated = matches!(raw.read(&mut probe), Ok(n) if n > 0);
    Ok((classify(&buf), truncated))
}

fn category_to_json(
    path: &str,
    category: SniffCategory,
    truncated: bool,
    format: OutputFormat,
    quiet: bool,
    out: &mut Output,
) -> Result<serde_json::Value, String> {
    let mut obj = serde_json::Map::new();
    obj.insert("path".into(), serde_json::Value::String(path.to_string()));
    if truncated {
        obj.insert("truncated".into(), serde_json::Value::Bool(true));
    }
    match category {
        SniffCategory::Binary(m) => {
            obj.insert(
                "format".into(),
                serde_json::Value::String(m.as_str().into()),
            );
            obj.insert(
                "category".into(),
                serde_json::Value::String("binary".into()),
            );
            if format == OutputFormat::Human && !quiet {
                out.writeln(&format!("format: {}", m.as_str()))?;
            }
        }
        SniffCategory::Json(v) => {
            obj.insert("format".into(), serde_json::Value::String("json".into()));
            obj.insert("data".into(), v);
            if format == OutputFormat::Human && !quiet {
                out.writeln(
                    &serde_json::to_string_pretty(&obj["data"]).map_err(|e| e.to_string())?,
                )?;
            }
        }
        SniffCategory::GeoJson(geo) => {
            obj.insert("format".into(), serde_json::Value::String("geojson".into()));
            obj.insert("valid".into(), serde_json::Value::Bool(true));
            let data = serde_json::to_value(&geo).map_err(|e| e.to_string())?;
            obj.insert("data".into(), data.clone());
            if format == OutputFormat::Human && !quiet {
                out.writeln(&serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?)?;
            }
        }
        SniffCategory::Jsonl(records) => {
            obj.insert("format".into(), serde_json::Value::String("jsonl".into()));
            obj.insert("records".into(), serde_json::Value::Number(records.into()));
            if format == OutputFormat::Human && !quiet {
                out.writeln(&format!("format: jsonl\nrecords: {records}"))?;
            }
        }
        SniffCategory::Logfmt(pairs) => {
            obj.insert("format".into(), serde_json::Value::String("logfmt".into()));
            let fields: Vec<serde_json::Value> = pairs
                .iter()
                .map(|(k, v)| {
                    let mut m = serde_json::Map::new();
                    m.insert("key".into(), serde_json::Value::String(k.clone()));
                    m.insert("value".into(), serde_json::Value::String(v.clone()));
                    serde_json::Value::Object(m)
                })
                .collect();
            obj.insert("fields".into(), serde_json::Value::Array(fields));
            if format == OutputFormat::Human && !quiet {
                out.writeln("format: logfmt")?;
                for (k, v) in &pairs {
                    out.writeln(&format!("{k} = {v}"))?;
                }
            }
        }
        SniffCategory::Unknown => {
            obj.insert("format".into(), serde_json::Value::String("unknown".into()));
            if format == OutputFormat::Human && !quiet {
                out.writeln("format: unknown")?;
            }
        }
    }
    Ok(serde_json::Value::Object(obj))
}

fn classify(buf: &[u8]) -> SniffCategory {
    // Binary detection runs FIRST, on the raw bytes. Many binary containers
    // (tar, PDF, ZIP, ...) are ASCII-clean in their header region, so a purely
    // text-based classifier would mislabel them as `unknown`.
    if let Some(m) = tpt_mime_pure::detect(buf) {
        // JSON is recognised by content, but a GeoJSON document is also valid
        // JSON: refine a `application/json` hit into `geojson` when the document
        // parses as GeoJSON, otherwise keep it as plain `json`. This keeps
        // `{"type":"Point","coordinates":[1,2]}` distinct from `{"a":1}`.
        if m == tpt_mime_pure::MimeType::Json {
            if let Ok(text) = std::str::from_utf8(buf) {
                let trimmed = text.trim();
                if let Ok(geo) = tpt_geo_geojson::parse(trimmed) {
                    return SniffCategory::GeoJson(geo);
                }
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
                    return SniffCategory::Json(value);
                }
            }
        }
        return SniffCategory::Binary(m);
    }

    let text = match std::str::from_utf8(buf) {
        Ok(t) => t.trim(),
        Err(_) => return SniffCategory::Unknown,
    };

    let looks_like_json_doc = text.starts_with('{') || text.starts_with('[');
    if looks_like_json_doc {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
            if let Ok(geo) = tpt_geo_geojson::parse(text) {
                return SniffCategory::GeoJson(geo);
            }
            return SniffCategory::Json(value);
        }
    }

    let non_empty: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if non_empty.is_empty() {
        return SniffCategory::Unknown;
    }

    let all_json = non_empty.iter().all(|l| {
        serde_json::from_str::<serde_json::Value>(l)
            .map(|v| {
                matches!(
                    v,
                    serde_json::Value::Object(_) | serde_json::Value::Array(_)
                )
            })
            .unwrap_or(false)
    });
    if all_json {
        return SniffCategory::Jsonl(non_empty.len());
    }

    let first = non_empty[0];
    if first.contains('=') {
        if let Ok(pairs) = tpt_logfmt_parse::parse_to_pairs(first) {
            if !pairs.is_empty() {
                return SniffCategory::Logfmt(pairs);
            }
        }
    }

    SniffCategory::Unknown
}

/// A normalised NDJSON envelope produced by `sniff --router` for a single input.
/// One such line is emitted per input file, suitable for piping into `jq`/`mlr`.
fn router_envelope(path: &str) -> Result<String, String> {
    let bytes = slurp_bytes(path)?;
    let mut obj = serde_json::Map::new();
    obj.insert("source".into(), serde_json::Value::String(path.to_string()));
    obj.insert(
        "bytes".into(),
            serde_json::Value::Number((bytes.len() as u64).into()),
    );

    // Run MIME detection first (on raw bytes); only fall through to text
    // classification when detection yields nothing.
    let (detected, envelope, extra) = if let Some(m) = tpt_mime_pure::detect(&bytes) {
        if m == tpt_mime_pure::MimeType::Json {
            dispatch_text(&bytes, &m)
        } else {
            (
                Some(m.as_str().to_string()),
                "binary".to_string(),
                serde_json::Map::new(),
            )
        }
    } else {
        dispatch_text(&bytes, &tpt_mime_pure::MimeType::Text)
    };

    obj.insert(
        "detected".into(),
        detected
            .map(serde_json::Value::String)
            .unwrap_or(serde_json::Value::Null),
    );
    obj.insert("envelope".into(), serde_json::Value::String(envelope));
    for (k, v) in extra {
        obj.insert(k, v);
    }
    serde_json::to_string(&serde_json::Value::Object(obj)).map_err(|e| e.to_string())
}

fn envelope_unknown(envelope_json: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(envelope_json)
        .map(|v| v["envelope"] == serde_json::Value::String("unknown".into()))
        .unwrap_or(false)
}

/// Given the raw bytes and an already-detected MIME type, route to the matching
/// parser and compute the envelope `--extra` summary fields.
fn dispatch_text(
    bytes: &[u8],
    detected: &tpt_mime_pure::MimeType,
) -> (
    Option<String>,
    String,
    serde_json::Map<String, serde_json::Value>,
) {
    let text = match std::str::from_utf8(bytes) {
        Ok(t) => t,
        Err(_) => return (None, "unknown".to_string(), serde_json::Map::new()),
    };
    let trimmed = text.trim();
    let mut extra = serde_json::Map::new();

    // GeoJSON / JSON document?
    if let Ok(geo) = tpt_geo_geojson::parse(trimmed) {
        let features = match &geo {
            tpt_geo_geojson::GeoJson::FeatureCollection(fc) => fc.features.len() as u64,
            tpt_geo_geojson::GeoJson::Feature(_) => 1,
            _ => 0,
        };
        extra.insert("features".into(), serde_json::Value::Number(features.into()));
        return (
            Some("application/geo+json".to_string()),
            "geojson".to_string(),
            extra,
        );
    }
    if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
        extra.insert("value_count".into(), serde_json::Value::Number(1.into()));
        return (
            Some("application/json".to_string()),
            "json".to_string(),
            extra,
        );
    }

    let non_empty: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if non_empty.is_empty() {
        return (None, "unknown".to_string(), extra);
    }

    let all_json = non_empty.iter().all(|l| {
        serde_json::from_str::<serde_json::Value>(l)
            .map(|v| matches!(v, serde_json::Value::Object(_) | serde_json::Value::Array(_)))
            .unwrap_or(false)
    });
    if all_json {
        let mut reader = BufReader::new(bytes);
        let records = tpt_jsonl_stream::parse_jsonl(&mut reader)
            .filter(|r| r.is_ok())
            .count() as u64;
        extra.insert("records".into(), serde_json::Value::Number(records.into()));
        return (
            Some("application/x-ndjson".to_string()),
            "jsonl".to_string(),
            extra,
        );
    }

    let first = non_empty[0];
    if first.contains('=') {
        if let Ok(pairs) = tpt_logfmt_parse::parse_to_pairs(first) {
            if !pairs.is_empty() {
                let lines = tpt_logfmt_parse::parse_logfmt_lines(BufReader::new(bytes))
                    .filter_map(|r| r.ok())
                    .count() as u64;
                extra.insert("lines".into(), serde_json::Value::Number(lines.into()));
                return (
                    Some("application/x-logfmt".to_string()),
                    "logfmt".to_string(),
                    extra,
                );
            }
        }
    }

    // Plain text: report line count.
    extra.insert(
        "lines".into(),
        serde_json::Value::Number((non_empty.len() as u64).into()),
    );
    let _ = detected;
    (
        Some(tpt_mime_pure::MimeType::Text.as_str().to_string()),
        "text".to_string(),
        extra,
    )
}

/// Read an entire input (path or `-` for stdin) into a byte buffer.
fn slurp_bytes(path: &str) -> Result<Vec<u8>, String> {
    let mut reader = open_read(path)?;
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).map_err(|e| e.to_string())?;
    Ok(buf)
}

/// Read the first line from stdin (used when a subcommand receives `-` as its
/// primary input, e.g. `tpt cron -`).
fn read_stdin_line() -> Result<String, String> {
    let mut s = String::new();
    std::io::stdin()
        .read_line(&mut s)
        .map_err(|e| e.to_string())?;
    Ok(s.trim().to_string())
}

/// Expand a list of path arguments, recursing into directories when `-r` is set.
/// `-` (stdin) is preserved verbatim. A directory without `-r` is an error.
fn expand_paths(paths: &[String], recursive: bool) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for p in paths {
        if p == "-" {
            out.push(p.clone());
            continue;
        }
        let meta = std::fs::metadata(p).map_err(|e| format!("{}: {}", p, e))?;
        if meta.is_file() {
            out.push(p.clone());
        } else if meta.is_dir() {
            if !recursive {
                return Err(format!(
                    "{p} is a directory (use -r/--recursive to include its files)"
                ));
            }
            walk_dir(p, &mut out)?;
        }
    }
    Ok(out)
}

fn walk_dir(dir: &str, out: &mut Vec<String>) -> Result<(), String> {
    for entry in std::fs::read_dir(dir).map_err(|e| format!("{dir}: {e}"))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let meta = entry.metadata().map_err(|e| e.to_string())?;
        if meta.is_file() {
            out.push(path.to_string_lossy().to_string());
        } else if meta.is_dir() {
            walk_dir(&path.to_string_lossy(), out)?;
        }
    }
    Ok(())
}

/// Verbose cron explanation: expanded field sets, the next 5 runs, and any DST
/// transitions crossed between consecutive runs.
fn explain_output(
    parsed: &tpt_cron_parse::CronExpr,
    expr: &str,
    timezone: &str,
    format: OutputFormat,
    out: &mut Output,
    quiet: bool,
) -> Result<(), String> {
    let tz = parse_tz(timezone)?;
    let human = parsed.to_human_readable();

    let expand = |f: &tpt_cron_parse::CronField, min: u8, max: u8| -> serde_json::Value {
        serde_json::Value::Array(
            f.expand(min, max)
                .into_iter()
                .map(serde_json::Value::from)
                .collect(),
        )
    };
    let seconds = parsed
        .seconds
        .as_ref()
        .map(|s| expand(s, 0, 59))
        .unwrap_or(serde_json::Value::Null);
    let mut fields = serde_json::Map::new();
    fields.insert("seconds".into(), seconds);
    fields.insert("minutes".into(), expand(&parsed.minutes, 0, 59));
    fields.insert("hours".into(), expand(&parsed.hours, 0, 23));
    fields.insert("day_of_month".into(), expand(&parsed.dom, 1, 31));
    fields.insert("month".into(), expand(&parsed.month, 1, 12));
    fields.insert("day_of_week".into(), expand(&parsed.dow, 0, 6));

    // Compute the next 5 runs using the timezone-aware scheduler, recording
    // each run's offset in the chosen timezone so DST transitions can be seen.
    let start = tz.now();
    let mut runs: Vec<DateTime<Utc>> = Vec::new();
    let mut offsets: Vec<String> = Vec::new();
    let mut cur = match &start {
        Now::Utc(t) => *t,
        Now::Local(t) => t.with_timezone(&Utc),
        Now::Offset(t) => t.with_timezone(&Utc),
    };
    let tz_offset = |utc: DateTime<Utc>| -> String {
        match &tz {
            TzSpec::Utc => "+00:00".to_string(),
            TzSpec::Local => Local.from_utc_datetime(&utc.naive_utc()).offset().to_string(),
            TzSpec::Offset(f) => f.to_string(),
        }
    };
    for _ in 0..5 {
        let next: Option<DateTime<Utc>> = match &tz {
            TzSpec::Utc => parsed.next_after_tz(cur),
            TzSpec::Local => parsed
                .next_after_tz(cur.with_timezone(&Local))
                .map(|n| n.with_timezone(&Utc)),
            TzSpec::Offset(f) => parsed
                .next_after_tz(cur.with_timezone(f))
                .map(|n| n.with_timezone(&Utc)),
        };
        match next {
            Some(n) => {
                let utc = n.with_timezone(&Utc);
                cur = utc;
                runs.push(utc);
                offsets.push(tz_offset(utc));
            }
            None => break,
        }
    }

    // Detect DST transitions: consecutive runs whose timezone offset differs.
    let mut dst: Vec<serde_json::Value> = Vec::new();
    for w in runs.windows(2).enumerate() {
        let (i, pair) = w;
        let (a, b) = (pair[0], pair[1]);
        if offsets[i] != offsets[i + 1] {
            let mut t = serde_json::Map::new();
            t.insert("after".into(), serde_json::Value::String(a.to_rfc3339()));
            t.insert("before".into(), serde_json::Value::String(b.to_rfc3339()));
            t.insert("from_offset".into(), serde_json::Value::String(offsets[i].clone()));
            t.insert("to_offset".into(), serde_json::Value::String(offsets[i + 1].clone()));
            dst.push(serde_json::Value::Object(t));
        }
    }

    let runs_json: Vec<serde_json::Value> = runs
        .iter()
        .map(|r| serde_json::Value::String(r.to_rfc3339()))
        .collect();

    if format == OutputFormat::Json {
        let mut obj = serde_json::Map::new();
        obj.insert("ok".into(), serde_json::Value::Bool(true));
        obj.insert("expression".into(), serde_json::Value::String(expr.to_string()));
        obj.insert("human".into(), serde_json::Value::String(human.clone()));
        obj.insert("fields".into(), serde_json::Value::Object(fields));
        obj.insert("next_runs".into(), serde_json::Value::Array(runs_json));
        obj.insert("dst_transitions".into(), serde_json::Value::Array(dst));
        emit_json_or(
            out,
            format,
            serde_json::Value::Object(obj),
            || human.clone(),
            quiet,
        )
    } else {
        if quiet {
            return Ok(());
        }
        out.writeln(&out.paint(&format!("cron: {human}"), "1;36"))?;
        out.writeln("expanded fields:")?;
        for (k, v) in &fields {
            let list = match v {
                serde_json::Value::Array(a) => a
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
                _ => "—".to_string(),
            };
            out.writeln(&format!("  {k}: [{list}]"))?;
        }
        out.writeln("next 5 runs:")?;
        for (i, r) in runs.iter().enumerate() {
            out.writeln(&format!("  {}: {}", i + 1, r.to_rfc3339()))?;
        }
        if dst.is_empty() {
            out.writeln("dst transitions crossed: none")?;
        } else {
            out.writeln("dst transitions crossed:")?;
            for t in &dst {
                out.writeln(&format!(
                    "  {} -> {} ({} to {})",
                    t["after"], t["before"], t["from_offset"], t["to_offset"]
                ))?;
            }
        }
        out.flush()
    }
}

/// Summary statistics for a JSON Lines input: record/error counts, a per-line
/// error histogram, throughput (MB/s), and field cardinality.
fn jsonl_stats(
    paths: &[String],
    recursive: bool,
    format: OutputFormat,
    out: &mut Output,
    quiet: bool,
) -> Result<(), String> {
    use std::collections::{HashMap, HashSet};
    let expanded = expand_paths(paths, recursive)?;
    let mut last: Result<(), String> = Ok(());
    for path in &expanded {
        let bytes = slurp_bytes(path)?;
        let start = Instant::now();
        let reader = BufReader::new(bytes.as_slice());
        let mut records = 0u64;
        let mut errors = 0u64;
        let mut error_lines: Vec<u64> = Vec::new();
        let mut fields: HashMap<String, (u64, HashSet<String>)> = HashMap::new();
        for res in tpt_jsonl_stream::parse_jsonl(reader) {
            match res {
                Ok(v) => {
                    records += 1;
                    if let serde_json::Value::Object(m) = &v {
                        for (k, val) in m {
                            let e = fields.entry(k.clone()).or_insert((0, HashSet::new()));
                            e.0 += 1;
                            e.1.insert(val.to_string());
                        }
                    }
                }
                Err(e) => {
                    errors += 1;
                    error_lines.push(e.line);
                }
            }
        }
        let elapsed = start.elapsed();
        let mb_per_s = (bytes.len() as f64 / 1e6) / elapsed.as_secs_f64().max(1e-9);

        let mut field_json = serde_json::Map::new();
        let mut field_names: Vec<&String> = fields.keys().collect();
        field_names.sort();
        for name in field_names {
            let (count, card) = &fields[name];
            let mut fm = serde_json::Map::new();
            fm.insert("count".into(), serde_json::Value::Number((*count).into()));
            fm.insert(
                "cardinality".into(),
                serde_json::Value::Number((card.len() as u64).into()),
            );
            field_json.insert(name.clone(), serde_json::Value::Object(fm));
        }

        let mut obj = serde_json::Map::new();
        obj.insert("ok".into(), serde_json::Value::Bool(true));
        obj.insert("path".into(), serde_json::Value::String(path.clone()));
        obj.insert("bytes".into(), serde_json::Value::Number(bytes.len().into()));
        obj.insert("elapsed_ms".into(), serde_json::Value::Number((elapsed.as_millis() as u64).into()));
        obj.insert(
            "mb_per_s".into(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(mb_per_s).unwrap_or(serde_json::Number::from(0)),
            ),
        );
        obj.insert("records".into(), serde_json::Value::Number(records.into()));
        obj.insert("errors".into(), serde_json::Value::Number(errors.into()));
        obj.insert(
            "error_lines".into(),
            serde_json::Value::Array(
                error_lines
                    .iter()
                    .map(|l| serde_json::Value::Number((*l).into()))
                    .collect(),
            ),
        );
        obj.insert("fields".into(), serde_json::Value::Object(field_json));
    last = emit_json_or(
        out,
        format,
        serde_json::Value::Object(obj),
        || {
            format!(
                "{}: {} records, {} errors, {:.2} MB/s, {} fields",
                    path,
                    records,
                    errors,
                    mb_per_s,
                    fields.len()
                )
            },
            quiet,
        );
    }
    last
}

/// Summary statistics for logfmt input: line/error counts, error histogram,
/// throughput, and field cardinality.
fn logfmt_stats(
    lines: &[String],
    format: OutputFormat,
    out: &mut Output,
    quiet: bool,
) -> Result<(), String> {
    use std::collections::{HashMap, HashSet};
    // Gather all lines: `-` reads stdin, otherwise use the provided arguments.
    let mut all: Vec<String> = Vec::new();
    for l in lines {
        if l == "-" {
            let stdin = std::io::stdin();
            for line in stdin.lock().lines() {
                all.push(line.map_err(|e| e.to_string())?);
            }
        } else {
            all.push(l.clone());
        }
    }
    let joined = all.join("\n");
    let bytes = joined.as_bytes();
    let start = Instant::now();
    let reader = BufReader::new(bytes);
    let mut records = 0u64;
    let mut errors = 0u64;
    let mut error_lines: Vec<u64> = Vec::new();
    let mut fields: HashMap<String, (u64, HashSet<String>)> = HashMap::new();
    for res in tpt_logfmt_parse::parse_logfmt_lines(reader) {
        match res {
            Ok(pairs) => {
                records += 1;
                for (k, v) in pairs {
                    let e = fields.entry(k).or_insert((0, HashSet::new()));
                    e.0 += 1;
                    e.1.insert(v);
                }
            }
            Err(e) => {
                errors += 1;
                error_lines.push(e.line);
            }
        }
    }
    let elapsed = start.elapsed();
    let mb_per_s = (bytes.len() as f64 / 1e6) / elapsed.as_secs_f64().max(1e-9);

    let mut field_json = serde_json::Map::new();
    let mut field_names: Vec<&String> = fields.keys().collect();
    field_names.sort();
    for name in field_names {
        let (count, card) = &fields[name];
        let mut fm = serde_json::Map::new();
        fm.insert("count".into(), serde_json::Value::Number((*count).into()));
        fm.insert(
            "cardinality".into(),
            serde_json::Value::Number((card.len() as u64).into()),
        );
        field_json.insert(name.clone(), serde_json::Value::Object(fm));
    }

    let mut obj = serde_json::Map::new();
    obj.insert("ok".into(), serde_json::Value::Bool(true));
    obj.insert("bytes".into(), serde_json::Value::Number(bytes.len().into()));
    obj.insert("elapsed_ms".into(), serde_json::Value::Number((elapsed.as_millis() as u64).into()));
    obj.insert(
        "mb_per_s".into(),
        serde_json::Value::Number(
            serde_json::Number::from_f64(mb_per_s).unwrap_or(serde_json::Number::from(0)),
        ),
    );
    obj.insert("records".into(), serde_json::Value::Number(records.into()));
    obj.insert("errors".into(), serde_json::Value::Number(errors.into()));
    obj.insert(
        "error_lines".into(),
        serde_json::Value::Array(
            error_lines
                .iter()
                .map(|l| serde_json::Value::Number((*l).into()))
                .collect(),
        ),
    );
    obj.insert("fields".into(), serde_json::Value::Object(field_json));
    emit_json_or(
        out,
        format,
        serde_json::Value::Object(obj),
        || {
            format!(
                "{} logfmt records, {} errors, {:.2} MB/s, {} fields",
                records,
                errors,
                mb_per_s,
                fields.len()
            )
        },
        quiet,
    )
}

/// Follow a JSON Lines file and print new records as they are appended.
fn watch_jsonl(
    paths: &[String],
    recursive: bool,
    format: OutputFormat,
    out: &mut Output,
    quiet: bool,
) -> Result<(), String> {
    let expanded = expand_paths(paths, recursive)?;
    let path = match expanded.first() {
        Some(p) => p,
        None => return Err("watch mode requires at least one file path".into()),
    };
    if path == "-" {
        return Err("watch mode requires a file path, not stdin".into());
    }
    {
        let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        file.seek(std::io::SeekFrom::End(0)).map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(file);
        let mut buf = String::new();
        loop {
            buf.clear();
            match reader.read_line(&mut buf) {
                Ok(0) => {
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
                Ok(_) => {
                    let trimmed = buf.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    match serde_json::from_str::<serde_json::Value>(trimmed) {
                        Ok(v) => {
                            let obj = || {
                                let mut o = serde_json::Map::new();
                                o.insert("ok".into(), serde_json::Value::Bool(true));
                                o.insert("path".into(), serde_json::Value::String(path.clone()));
                                o.insert("record".into(), v);
                                serde_json::Value::Object(o)
                            };
                            let human = || trimmed.to_string();
                            emit_json_or(out, format, obj(), human, quiet)?;
                        }
                        Err(e) => {
                            emit_err(format, &format!("{}: parse error: {e}", path));
                        }
                    }
                }
                Err(e) => return Err(e.to_string()),
            }
        }
    }
    Ok(())
}
