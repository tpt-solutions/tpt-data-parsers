//! Golden output for every parser. Regenerate with `TPT_UPDATE_GOLDEN=1 cargo test`.

use std::fmt::Write as _;

use tpt_testkit::tpt_assert_golden;

#[test]
fn logfmt_pairs() {
    let mut out = String::new();
    for line in [
        "level=info msg=\"hello world\" count=3",
        "bare key=value",
        "msg=\"escaped \\\" quote\" empty=",
        "=novalue",
        "unterminated=\"oops",
    ] {
        match tpt_logfmt_parse::parse_to_pairs(line) {
            Ok(pairs) => writeln!(out, "{line} => {pairs:?}").unwrap(),
            Err(e) => writeln!(out, "{line} => error: {e}").unwrap(),
        }
    }
    tpt_assert_golden!(out, "logfmt_pairs.expected.txt");
}

#[test]
fn cron_expressions() {
    let mut out = String::new();
    for expr in [
        "* * * * *",
        "*/15 0 1,15 * 1-5",
        "0 0 1 1 *",
        "30 4 * * 0",
        "0 */2 * * * *",
        "61 * * * *",
        "* * *",
        "",
    ] {
        match tpt_cron_parse::CronExpr::parse(expr) {
            Ok(parsed) => writeln!(out, "{expr:?} => {}", parsed.to_human_readable()).unwrap(),
            Err(e) => writeln!(out, "{expr:?} => error: {e}").unwrap(),
        }
    }
    tpt_assert_golden!(out, "cron_expressions.expected.txt");
}

#[test]
fn mime_detection() {
    let samples: [(&str, &[u8]); 6] = [
        ("png", b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR"),
        ("pdf", b"%PDF-1.7\n"),
        ("gzip", b"\x1f\x8b\x08\0"),
        ("jpeg", b"\xff\xd8\xff\xe0"),
        ("text", b"just some text"),
        ("empty", b""),
    ];
    let mut out = String::new();
    for (name, bytes) in samples {
        writeln!(out, "{name}: {:?}", tpt_mime_pure::detect(bytes)).unwrap();
    }
    for ext in ["png", "pdf", "json", "nope"] {
        writeln!(out, ".{ext}: {:?}", tpt_mime_pure::detect_by_extension(ext)).unwrap();
    }
    tpt_assert_golden!(out, "mime_detection.expected.txt");
}

#[test]
fn jsonl_records() {
    let input = include_str!("../../tpt-jsonl-stream/tests/fixtures/sample.jsonl");
    let mut out = String::new();
    for (i, record) in tpt_jsonl_stream::parse_jsonl(input.as_bytes())
        .take(5)
        .enumerate()
    {
        match record {
            Ok(value) => writeln!(out, "{i}: {value}").unwrap(),
            Err(e) => writeln!(out, "{i}: error: {e}").unwrap(),
        }
    }
    let broken = "{\"a\":1}\n{broken\n\n[1,2]\n";
    for (i, record) in tpt_jsonl_stream::parse_jsonl(broken.as_bytes()).enumerate() {
        match record {
            Ok(value) => writeln!(out, "broken {i}: {value}").unwrap(),
            Err(e) => writeln!(out, "broken {i}: error: {e}").unwrap(),
        }
    }
    tpt_assert_golden!(out, "jsonl_records.expected.txt");
}

#[test]
fn geojson_fixtures() {
    let mut out = String::new();
    for (name, text) in [
        (
            "valid",
            include_str!("../../tpt-geo-geojson/tests/fixtures/valid.geojson"),
        ),
        (
            "malformed_coords",
            include_str!("../../tpt-geo-geojson/tests/fixtures/malformed_coords.geojson"),
        ),
        (
            "unclosed_polygon",
            include_str!("../../tpt-geo-geojson/tests/fixtures/unclosed_polygon.geojson"),
        ),
    ] {
        match tpt_geo_geojson::parse(text) {
            Ok(parsed) => match parsed.validate() {
                Ok(()) => writeln!(
                    out,
                    "{name}: valid\n{}",
                    tpt_geo_geojson::to_json(&parsed).unwrap()
                ),
                Err(e) => writeln!(out, "{name}: invalid: {e}"),
            },
            Err(e) => writeln!(out, "{name}: parse error: {e}"),
        }
        .unwrap();
    }
    tpt_assert_golden!(out, "geojson_fixtures.expected.txt");
}
