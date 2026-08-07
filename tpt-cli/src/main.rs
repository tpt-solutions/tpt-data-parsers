use clap::{Parser, Subcommand};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "tpt", version, about = "TPT data parser CLI")]
struct Cli {
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
    },
    /// Detect a file's MIME type from its bytes (falls back to extension)
    Mime {
        /// Path to the file
        path: PathBuf,
    },
    /// Validate a GeoJSON file
    Geojson {
        /// Path to the GeoJSON file
        path: PathBuf,
    },
    /// Parse a single logfmt line
    Logfmt {
        /// The logfmt line to parse
        line: String,
    },
    /// Count the records in a JSON Lines file
    Jsonl {
        /// Path to the .jsonl file
        path: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Cron { expr, next } => {
            let parsed = tpt_cron_parse::CronExpr::parse(&expr).map_err(|e| e.to_string())?;
            println!("{}", parsed.to_human_readable());
            if next {
                use chrono::{Timelike, Utc};
                let after = Utc::now()
                    .with_second(0)
                    .and_then(|t| t.with_nanosecond(0))
                    .unwrap();
                match parsed.next_after(after) {
                    Some(t) => println!("next run: {t}"),
                    None => println!("next run: none within 4 years"),
                }
            }
            Ok(())
        }
        Command::Mime { path } => {
            let mut buf = Vec::new();
            {
                let mut f = std::fs::File::open(&path).map_err(|e| e.to_string())?;
                f.read_to_end(&mut buf).map_err(|e| e.to_string())?;
            }
            let mime = tpt_mime_pure::detect(&buf).or_else(|| {
                path.extension()
                    .and_then(|s| s.to_str())
                    .and_then(tpt_mime_pure::detect_by_extension)
            });
            match mime {
                Some(m) => {
                    println!("{}", m.as_str());
                    Ok(())
                }
                None => Err("unknown type".into()),
            }
        }
        Command::Geojson { path } => {
            let mut s = String::new();
            std::fs::File::open(&path)
                .map_err(|e| e.to_string())?
                .read_to_string(&mut s)
                .map_err(|e| e.to_string())?;
            tpt_geo_geojson::parse(&s).map_err(|e| e.to_string())?;
            println!("valid");
            Ok(())
        }
        Command::Logfmt { line } => {
            for pair in tpt_logfmt_parse::LogfmtParser::new(&line) {
                let (k, v) = pair.map_err(|e| e.to_string())?;
                println!("{k} = {v}");
            }
            Ok(())
        }
        Command::Jsonl { path } => {
            let f = std::fs::File::open(&path).map_err(|e| e.to_string())?;
            let reader = std::io::BufReader::new(f);
            let mut count = 0u64;
            for res in tpt_jsonl_stream::parse_jsonl(reader) {
                res.map_err(|e| e.to_string())?;
                count += 1;
            }
            println!("records: {count}");
            Ok(())
        }
    }
}
