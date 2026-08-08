use chrono::{DateTime, FixedOffset, Local, Utc};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::ExitCode;
use std::str::FromStr;

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
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Parse and describe a cron expression
    Cron {
        /// Cron expression (5 or 6 fields)
        expr: String,
        /// Also compute the next run time (needs the chrono feature)
        #[arg(long)]
        next: bool,
        /// Timezone for --next: UTC, local, or a fixed offset like +08:00.
        /// Affects when the schedule fires, not just how it is displayed.
        #[arg(long, short = 'z', default_value = "UTC")]
        timezone: String,
    },
    /// Detect a file's MIME type from its bytes (falls back to extension)
    Mime {
        /// One or more paths, or - for stdin
        #[arg(required = true)]
        paths: Vec<String>,
    },
    /// Validate a GeoJSON document
    Geojson {
        /// One or more paths, or - for stdin
        #[arg(required = true)]
        paths: Vec<String>,
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
    },
    /// Detect a path's type and auto-dispatch to the matching parser
    Sniff {
        /// One or more paths, or - for stdin
        #[arg(required = true)]
        paths: Vec<String>,
    },
    /// Generate shell completion scripts
    Completions {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let format = if cli.json {
        OutputFormat::Json
    } else {
        cli.format
    };
    match run(cli, format) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            emit_err(format, &e);
            ExitCode::FAILURE
        }
    }
}

fn emit(line: &str) -> Result<(), String> {
    let mut out = std::io::stdout().lock();
    match writeln!(out, "{line}") {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => Ok(()),
        Err(e) => Err(e.to_string()),
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
    format: OutputFormat,
    value: serde_json::Value,
    human: impl FnOnce() -> String,
    quiet: bool,
) -> Result<(), String> {
    match format {
        OutputFormat::Json => {
            let s = serde_json::to_string(&value).map_err(|e| e.to_string())?;
            emit(&s)
        }
        OutputFormat::Human => {
            if quiet {
                Ok(())
            } else {
                emit(&human())
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

fn run(cli: Cli, format: OutputFormat) -> Result<(), String> {
    let quiet = cli.quiet;
    match cli.command {
        Command::Cron {
            expr,
            next,
            timezone,
        } => {
            let parsed = tpt_cron_parse::CronExpr::parse(&expr).map_err(|e| e.to_string())?;
            let human = parsed.to_human_readable();
            let next_run = if next {
                let tz = parse_tz(&timezone)?;
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
                    format,
                    serde_json::Value::Object(obj),
                    || human.clone(),
                    quiet,
                )
            } else {
                emit_json_or(
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
        Command::Mime { paths } => {
            let mut last = Ok(());
            for path in &paths {
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
                        emit_json_or(format, obj(), || human().clone(), quiet)
                    }
                    None => Err("unknown type".into()),
                };
            }
            last
        }
        Command::Geojson { paths } => {
            let mut last = Ok(());
            for path in &paths {
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
                last = emit_json_or(format, obj, || human.clone(), quiet);
            }
            last
        }
        Command::Logfmt { lines } => {
            // Expand "-" (stdin) into the actual lines so `-` can be batched.
            let mut expanded: Vec<String> = Vec::new();
            for l in &lines {
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
                last = emit_json_or(format, obj(), || human().clone(), quiet);
            }
            last
        }
        Command::Jsonl { paths } => {
            let mut last = Ok(());
            for path in &paths {
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
                last = emit_json_or(format, obj(), || human().clone(), quiet);
            }
            last
        }
        Command::Sniff { paths } => {
            // A single JSON envelope combining every inspected path, so the
            // output is stable in both human and JSON modes regardless of input.
            // A path that classifies as `unknown` is still a valid result, but
            // mirrors `mime` (which exits 1 when nothing is detected) by making
            // the process exit non-zero so CI/pipelines can treat it as a failure.
            let mut results: Vec<serde_json::Value> = Vec::new();
            let mut any_unknown = false;
            for path in &paths {
                let (category, truncated) = classify_path(path)?;
                if matches!(category, SniffCategory::Unknown) {
                    any_unknown = true;
                }
                results.push(category_to_json(path, category, truncated, format, quiet)?);
            }
            if format == OutputFormat::Json {
                let mut arr = serde_json::Map::new();
                arr.insert("ok".into(), serde_json::Value::Bool(!any_unknown));
                arr.insert("results".into(), serde_json::Value::Array(results));
                emit(
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
            clap_complete::generate(shell, &mut cmd, "tpt", &mut buf);
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
            .ok_or_else(|| "none within 4 years".to_string()),
        Now::Local(t) => parsed
            .next_after_tz(t)
            .map(|t| t.to_rfc3339())
            .ok_or_else(|| "none within 4 years".to_string()),
        Now::Offset(t) => parsed
            .next_after_tz(t)
            .map(|t| t.to_string())
            .ok_or_else(|| "none within 4 years".to_string()),
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
                emit(&format!("format: {}", m.as_str()))?;
            }
        }
        SniffCategory::Json(v) => {
            obj.insert("format".into(), serde_json::Value::String("json".into()));
            obj.insert("data".into(), v);
            if format == OutputFormat::Human && !quiet {
                emit(&serde_json::to_string_pretty(&obj["data"]).map_err(|e| e.to_string())?)?;
            }
        }
        SniffCategory::GeoJson(geo) => {
            obj.insert("format".into(), serde_json::Value::String("geojson".into()));
            obj.insert("valid".into(), serde_json::Value::Bool(true));
            let data = serde_json::to_value(&geo).map_err(|e| e.to_string())?;
            obj.insert("data".into(), data.clone());
            if format == OutputFormat::Human && !quiet {
                emit(&serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?)?;
            }
        }
        SniffCategory::Jsonl(records) => {
            obj.insert("format".into(), serde_json::Value::String("jsonl".into()));
            obj.insert("records".into(), serde_json::Value::Number(records.into()));
            if format == OutputFormat::Human && !quiet {
                emit(&format!("format: jsonl\nrecords: {records}"))?;
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
                emit("format: logfmt")?;
                for (k, v) in &pairs {
                    emit(&format!("{k} = {v}"))?;
                }
            }
        }
        SniffCategory::Unknown => {
            obj.insert("format".into(), serde_json::Value::String("unknown".into()));
            if format == OutputFormat::Human && !quiet {
                emit("format: unknown")?;
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
