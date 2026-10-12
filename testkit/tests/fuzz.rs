//! In-process fuzzing of every parser with `tpt_testkit::fuzz::Engine`.
//!
//! Environment knobs (all optional): `TPT_FUZZ_ITERATIONS` (default 3000 per parser),
//! `TPT_FUZZ_SEED`, `TPT_FUZZ_CORPUS` (a directory; one subdirectory per parser),
//! `TPT_FUZZ_CRASHES`, `TPT_FUZZ_REPORT` (one `<stem>-<parser>.json` per parser).

use std::env;
use std::path::PathBuf;

use tpt_testkit::fuzz::{self, Engine};

fn env_u64(name: &str, default: u64) -> u64 {
    env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Runs one parser through the engine and fails if any input panics.
fn fuzz_parser(name: &str, seeds: Vec<Vec<u8>>, dictionary: &[&[u8]], target: fn(&[u8])) {
    let corpus_dir = env::var_os("TPT_FUZZ_CORPUS").map(|d| PathBuf::from(d).join(name));
    let mut seeds = seeds;
    if let Some(dir) = corpus_dir.as_ref().filter(|dir| dir.is_dir()) {
        seeds.extend(fuzz::read_corpus(dir).expect("corpus directory is readable"));
    }
    let mut engine = Engine::new(env_u64("TPT_FUZZ_SEED", 1), seeds).with_dictionary(dictionary);
    // The engine learns from the features a target reports. Input length (log2 bucket) and first
    // byte give it a cheap spread without instrumentation.
    engine.run(env_u64("TPT_FUZZ_ITERATIONS", 3000), |data| {
        target(data);
        vec![
            u64::from(usize::BITS - data.len().leading_zeros()),
            0x100 | u64::from(data.first().copied().unwrap_or(0)),
        ]
    });
    eprintln!(
        "{name}: {} executions, {} corpus entries, {} crashes",
        engine.executions(),
        engine.corpus().len(),
        engine.crashes().len()
    );
    if let Some(dir) = corpus_dir {
        fuzz::save_corpus(engine.corpus(), dir).expect("corpus is writable");
    }
    if let Some(dir) = env::var_os("TPT_FUZZ_CRASHES") {
        fuzz::save_failures(engine.crashes(), PathBuf::from(dir).join(name))
            .expect("crash directory is writable");
    }
    if let Some(path) = env::var_os("TPT_FUZZ_REPORT") {
        let path = PathBuf::from(path);
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("fuzz-report");
        fuzz::write_report(
            engine.crashes(),
            path.with_file_name(format!("{stem}-{name}.json")),
        )
        .expect("report path is writable");
    }
    let groups = fuzz::dedup(engine.crashes());
    assert!(
        groups.is_empty(),
        "{name}: {} crash group(s); first: {:?}",
        groups.len(),
        groups.first().map(|g| (&g.message, &g.smallest))
    );
}

fn text(data: &[u8]) -> Option<&str> {
    std::str::from_utf8(data).ok()
}

#[test]
fn logfmt() {
    fuzz_parser(
        "logfmt",
        vec![
            b"level=info msg=\"hello world\" n=3".to_vec(),
            b"a=\"b \\\" c\" d= e".to_vec(),
        ],
        &[b"=", b"\"", b"\\\"", b" ", b"\n"],
        |data| {
            if let Some(s) = text(data) {
                let _ = tpt_logfmt_parse::LogfmtParser::new(s).count();
                let _ = tpt_logfmt_parse::parse_to_pairs(s);
                let _ = tpt_logfmt_parse::parse_logfmt_lines(std::io::Cursor::new(data)).count();
            }
        },
    );
}

#[test]
fn cron() {
    fuzz_parser(
        "cron",
        vec![
            b"*/15 0 1,15 * 1-5".to_vec(),
            b"0 0 */2 * * *".to_vec(),
            b"@daily".to_vec(),
        ],
        &[
            b"*", b"/", b",", b"-", b"?", b"L", b"W", b"#", b"MON", b"JAN",
        ],
        |data| {
            if let Some(s) = text(data) {
                if let Ok(expr) = tpt_cron_parse::CronExpr::parse(s) {
                    let _ = expr.to_human_readable();
                    let _ = expr.minutes.expand(0, 59);
                }
            }
        },
    );
}

#[test]
fn mime() {
    fuzz_parser(
        "mime",
        vec![
            b"\x89PNG\r\n\x1a\n".to_vec(),
            b"%PDF-1.7\n".to_vec(),
            b"PK\x03\x04".to_vec(),
            b"\xff\xd8\xff".to_vec(),
        ],
        &[b"RIFF", b"WEBP", b"ftyp", b"GIF8", b"\x00\x00\x01\x00"],
        |data| {
            let _ = tpt_mime_pure::detect(data);
            let _ = tpt_mime_pure::detect_all(data).count();
            let _ = tpt_mime_pure::detect_with_confidence(data);
            if let Some(s) = text(data) {
                let _ = tpt_mime_pure::detect_by_extension(s);
            }
        },
    );
}

#[test]
fn jsonl() {
    fuzz_parser(
        "jsonl",
        vec![
            include_bytes!("../../tpt-jsonl-stream/tests/fixtures/sample.jsonl").to_vec(),
            b"{\"a\":1}\n[1,2]\n".to_vec(),
        ],
        fuzz::JSON_TOKENS,
        |data| {
            let _ = tpt_jsonl_stream::parse_jsonl(std::io::Cursor::new(data)).count();
        },
    );
}

#[test]
fn geojson() {
    fuzz_parser(
        "geojson",
        vec![
            include_bytes!("../../tpt-geo-geojson/tests/fixtures/valid.geojson").to_vec(),
            include_bytes!("../../tpt-geo-geojson/tests/fixtures/unclosed_polygon.geojson")
                .to_vec(),
        ],
        fuzz::JSON_TOKENS,
        |data| {
            if let Some(s) = text(data) {
                if let Ok(parsed) = tpt_geo_geojson::parse(s) {
                    let _ = parsed.validate();
                    let _ = tpt_geo_geojson::to_json(&parsed);
                }
            }
        },
    );
}
