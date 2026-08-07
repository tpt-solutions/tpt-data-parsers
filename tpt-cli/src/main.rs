use chrono::{DateTime, FixedOffset, Local, Timelike, Utc};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use std::io::{BufReader, Read, Write};
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
        /// Timezone for --next: UTC, local, or a fixed offset like +08:00
        #[arg(long, short = 'z', default_value = "UTC")]
        timezone: String,
    },
    /// Detect a file's MIME type from its bytes (falls back to extension)
    Mime {
        /// Path to the file, or - for stdin
        path: String,
    },
    /// Validate a GeoJSON document
    Geojson {
        /// Path to the GeoJSON file, or - for stdin
        path: String,
    },
    /// Parse a single logfmt line
    Logfmt {
        /// The logfmt line to parse
        line: String,
    },
    /// Count the records in a JSON Lines file
    Jsonl {
        /// Path to the .jsonl file, or - for stdin
        path: String,
    },
    /// Detect a path's type and auto-dispatch to the matching parser
    Sniff {
        /// Path to inspect, or - for stdin
        path: String,
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
            let _ = writeln!(std::io::stderr(), "error: {e}");
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

fn emit_json(value: &serde_json::Value) -> Result<(), String> {
    let s = serde_json::to_string(value).map_err(|e| e.to_string())?;
    emit(&s)
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
    fn now_utc(&self) -> DateTime<Utc> {
        match self {
            TzSpec::Utc => Utc::now(),
            TzSpec::Local => Local::now().with_timezone(&Utc),
            TzSpec::Offset(f) => Utc::now().with_timezone(f).with_timezone(&Utc),
        }
    }

    fn display(&self, dt: DateTime<Utc>) -> String {
        match self {
            TzSpec::Utc => dt.to_rfc3339(),
            TzSpec::Local => dt.with_timezone(&Local).to_rfc3339(),
            TzSpec::Offset(f) => dt.with_timezone(f).to_string(),
        }
    }
}

fn run(cli: Cli, format: OutputFormat) -> Result<(), String> {
    match cli.command {
        Command::Cron {
            expr,
            next,
            timezone,
        } => {
            let parsed = tpt_cron_parse::CronExpr::parse(&expr).map_err(|e| e.to_string())?;
            let human = parsed.to_human_readable();
            if format == OutputFormat::Json {
                let mut obj = serde_json::Map::new();
                obj.insert("expression".into(), serde_json::Value::String(expr));
                obj.insert("human".into(), serde_json::Value::String(human));
                if next {
                    let tz = parse_tz(&timezone)?;
                    let after = tz
                        .now_utc()
                        .with_second(0)
                        .and_then(|t| t.with_nanosecond(0))
                        .unwrap();
                    let next_run = parsed
                        .next_after(after)
                        .map(|t| tz.display(t))
                        .unwrap_or_else(|| "none within 4 years".to_string());
                    obj.insert("next".into(), serde_json::Value::String(next_run));
                }
                emit_json(&serde_json::Value::Object(obj))
            } else {
                emit(&human)?;
                if next {
                    let tz = parse_tz(&timezone)?;
                    let after = tz
                        .now_utc()
                        .with_second(0)
                        .and_then(|t| t.with_nanosecond(0))
                        .unwrap();
                    match parsed.next_after(after) {
                        Some(t) => emit(&format!("next run: {}", tz.display(t)))?,
                        None => emit("next run: none within 4 years")?,
                    }
                }
                Ok(())
            }
        }
        Command::Mime { path } => {
            let mime = detect_mime(&path)?;
            match mime {
                Some(m) => {
                    if format == OutputFormat::Json {
                        let mut obj = serde_json::Map::new();
                        obj.insert("mime".into(), serde_json::Value::String(m.as_str().into()));
                        obj.insert(
                            "extension".into(),
                            serde_json::Value::String(m.extension().into()),
                        );
                        emit_json(&serde_json::Value::Object(obj))
                    } else {
                        emit(m.as_str())
                    }
                }
                None => Err("unknown type".into()),
            }
        }
        Command::Geojson { path } => {
            let reader = open_read(&path)?;
            let geo = tpt_geo_geojson::parse_reader(reader).map_err(|e| e.to_string())?;
            if format == OutputFormat::Json {
                let data = serde_json::to_value(&geo).map_err(|e| e.to_string())?;
                let mut obj = serde_json::Map::new();
                obj.insert("valid".into(), serde_json::Value::Bool(true));
                obj.insert("data".into(), data);
                emit_json(&serde_json::Value::Object(obj))
            } else {
                emit("valid")
            }
        }
        Command::Logfmt { line } => {
            let map = tpt_logfmt_parse::parse_to_map(&line).map_err(|e| e.to_string())?;
            if format == OutputFormat::Json {
                let fields: serde_json::Map<String, serde_json::Value> = map
                    .into_iter()
                    .map(|(k, v)| (k, serde_json::Value::String(v)))
                    .collect();
                let mut obj = serde_json::Map::new();
                obj.insert("fields".into(), serde_json::Value::Object(fields));
                emit_json(&serde_json::Value::Object(obj))
            } else {
                let mut out = String::new();
                for (k, v) in map {
                    out.push_str(&format!("{k} = {v}\n"));
                }
                if out.is_empty() {
                    Ok(())
                } else {
                    emit(out.trim_end())
                }
            }
        }
        Command::Jsonl { path } => {
            let reader = BufReader::new(open_read(&path)?);
            let mut count = 0u64;
            for res in tpt_jsonl_stream::parse_jsonl(reader) {
                res.map_err(|e| e.to_string())?;
                count += 1;
            }
            if format == OutputFormat::Json {
                let mut obj = serde_json::Map::new();
                obj.insert("records".into(), serde_json::Value::Number(count.into()));
                emit_json(&serde_json::Value::Object(obj))
            } else {
                emit(&format!("records: {count}"))
            }
        }
        Command::Sniff { path } => run_sniff(&path, format),
        Command::Completions { shell } => {
            let mut cmd = Cli::command();
            clap_complete::generate(shell, &mut cmd, "tpt", &mut std::io::stdout());
            Ok(())
        }
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
        Ok(path
            .rsplit('.')
            .next()
            .and_then(tpt_mime_pure::detect_by_extension))
    }
}

enum SniffCategory {
    Binary(tpt_mime_pure::MimeType),
    Json(serde_json::Value),
    GeoJson(tpt_geo_geojson::GeoJson),
    Jsonl(usize),
    Logfmt(std::collections::HashMap<String, String>),
    Unknown,
}

fn run_sniff(path: &str, format: OutputFormat) -> Result<(), String> {
    let mut buf = Vec::new();
    open_read(path)?
        .take(1 << 20)
        .read_to_end(&mut buf)
        .map_err(|e| e.to_string())?;
    let category = classify(&buf);
    match category {
        SniffCategory::Binary(m) => {
            if format == OutputFormat::Json {
                let mut obj = serde_json::Map::new();
                obj.insert(
                    "format".into(),
                    serde_json::Value::String(m.as_str().into()),
                );
                obj.insert(
                    "category".into(),
                    serde_json::Value::String("binary".into()),
                );
                emit_json(&serde_json::Value::Object(obj))
            } else {
                emit(&format!("format: {}", m.as_str()))
            }
        }
        SniffCategory::Json(v) => {
            if format == OutputFormat::Json {
                let mut obj = serde_json::Map::new();
                obj.insert("format".into(), serde_json::Value::String("json".into()));
                obj.insert("data".into(), v);
                emit_json(&serde_json::Value::Object(obj))
            } else {
                let pretty = serde_json::to_string_pretty(&v).map_err(|e| e.to_string())?;
                emit(&pretty)
            }
        }
        SniffCategory::GeoJson(geo) => {
            if format == OutputFormat::Json {
                let data = serde_json::to_value(&geo).map_err(|e| e.to_string())?;
                let mut obj = serde_json::Map::new();
                obj.insert("format".into(), serde_json::Value::String("geojson".into()));
                obj.insert("valid".into(), serde_json::Value::Bool(true));
                obj.insert("data".into(), data);
                emit_json(&serde_json::Value::Object(obj))
            } else {
                let pretty = serde_json::to_string_pretty(&geo).map_err(|e| e.to_string())?;
                emit(&pretty)
            }
        }
        SniffCategory::Jsonl(records) => {
            if format == OutputFormat::Json {
                let mut obj = serde_json::Map::new();
                obj.insert("format".into(), serde_json::Value::String("jsonl".into()));
                obj.insert("records".into(), serde_json::Value::Number(records.into()));
                emit_json(&serde_json::Value::Object(obj))
            } else {
                emit(&format!("format: jsonl\nrecords: {records}"))
            }
        }
        SniffCategory::Logfmt(map) => {
            if format == OutputFormat::Json {
                let fields: serde_json::Map<String, serde_json::Value> = map
                    .into_iter()
                    .map(|(k, v)| (k, serde_json::Value::String(v)))
                    .collect();
                let mut obj = serde_json::Map::new();
                obj.insert("format".into(), serde_json::Value::String("logfmt".into()));
                obj.insert("fields".into(), serde_json::Value::Object(fields));
                emit_json(&serde_json::Value::Object(obj))
            } else {
                emit("format: logfmt")?;
                for (k, v) in map {
                    emit(&format!("{k} = {v}"))?;
                }
                Ok(())
            }
        }
        SniffCategory::Unknown => {
            if format == OutputFormat::Json {
                let mut obj = serde_json::Map::new();
                obj.insert("format".into(), serde_json::Value::String("unknown".into()));
                emit_json(&serde_json::Value::Object(obj))
            } else {
                emit("format: unknown")
            }
        }
    }
}

fn classify(buf: &[u8]) -> SniffCategory {
    let text = match std::str::from_utf8(buf) {
        Ok(t) => t.trim(),
        Err(_) => {
            return match tpt_mime_pure::detect(buf) {
                Some(m) => SniffCategory::Binary(m),
                None => SniffCategory::Unknown,
            };
        }
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

    let all_json = non_empty
        .iter()
        .all(|l| serde_json::from_str::<serde_json::Value>(l).is_ok());
    if all_json {
        return SniffCategory::Jsonl(non_empty.len());
    }

    let first = non_empty[0];
    if first.contains('=') {
        if let Ok(map) = tpt_logfmt_parse::parse_to_map(first) {
            if !map.is_empty() {
                return SniffCategory::Logfmt(map);
            }
        }
    }

    SniffCategory::Unknown
}
