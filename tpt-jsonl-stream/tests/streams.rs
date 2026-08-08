use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use tpt_jsonl_stream::{parse_jsonl, JsonlErrorKind};

#[derive(Deserialize, Debug, PartialEq)]
struct Record {
    id: u64,
    name: String,
    active: bool,
}

#[test]
fn integration_reads_100_line_fixture() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/sample.jsonl");
    let f = File::open(path).expect("fixture missing");
    let values: Vec<_> = parse_jsonl(BufReader::new(f))
        .collect::<Result<_, _>>()
        .expect("parse failed");
    assert_eq!(values.len(), 100);
    assert_eq!(values[0]["id"], 0);
    assert_eq!(values[99]["id"], 99);
}

#[test]
fn integration_first_item_has_expected_fields() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/sample.jsonl");
    let f = File::open(path).expect("fixture missing");
    let values: Vec<_> = parse_jsonl(BufReader::new(f))
        .collect::<Result<_, _>>()
        .unwrap();
    let first = &values[0];
    assert!(first.get("name").is_some());
    assert!(first.get("value").is_some());
    assert!(first.get("active").is_some());
}

#[test]
fn integration_error_carries_line_number() {
    let data = b"{\"ok\":true}\n{bad json\n{\"ok\":true}\n";
    let mut reader = parse_jsonl(BufReader::new(data.as_slice()));
    reader.next().unwrap().unwrap(); // line 1
    let err = reader.next().unwrap().unwrap_err();
    assert_eq!(err.line, 2);
    assert!(!err.to_string().is_empty());
}

#[test]
fn integration_blank_lines_in_fixture_skipped() {
    let data = b"\n\n{\"x\":1}\n\n{\"y\":2}\n\n";
    let values: Vec<_> = parse_jsonl(BufReader::new(data.as_slice()))
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(values.len(), 2);
}

#[test]
fn integration_typed_iterator_reads_fixture() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/sample.jsonl");
    let f = File::open(path).expect("fixture missing");
    let records: Vec<Record> = parse_jsonl(BufReader::new(f))
        .into_typed::<Record>()
        .collect::<Result<_, _>>()
        .expect("deserialize failed");
    assert_eq!(records.len(), 100);
    assert_eq!(records[0].id, 0);
    assert_eq!(records[0].name, "item_0");
    assert_eq!(records[99].id, 99);
}

#[test]
fn integration_over_long_line_does_not_buffer() {
    let huge = "x".repeat(1_000_000);
    let data = format!("{{\"a\":1}}\n\"{}\"\n{{\"c\":3}}\n", huge);
    let mut reader = parse_jsonl(BufReader::new(data.as_bytes())).with_max_line_length(1024);
    assert_eq!(reader.next().unwrap().unwrap()["a"], 1);
    let err = reader.next().unwrap().unwrap_err();
    assert_eq!(err.line, 2);
    assert!(matches!(
        err.kind,
        JsonlErrorKind::LineTooLong { limit: 1024 }
    ));
    assert_eq!(reader.next().unwrap().unwrap()["c"], 3);
    assert!(reader.next().is_none());
}

#[test]
fn integration_non_breaking_space_line_is_not_skipped() {
    let data = "{\"x\":1}\n\u{a0}\n{\"y\":2}\n";
    let results: Vec<_> = parse_jsonl(BufReader::new(data.as_bytes())).collect();
    assert_eq!(results.len(), 3);
    assert!(results[0].is_ok());
    assert_eq!(results[1].as_ref().unwrap_err().line, 2);
    assert!(results[2].is_ok());
}

/// A reader that always fails with a persistent I/O error must terminate, not
/// yield errors forever (regression for the infinite-loop-on-IO-error bug).
#[test]
fn integration_persistent_io_error_terminates() {
    use std::io::{Error, ErrorKind, Read};

    struct AlwaysFails;
    impl Read for AlwaysFails {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(Error::new(ErrorKind::Other, "boom"))
        }
    }
    impl std::io::BufRead for AlwaysFails {
        fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
            Err(Error::new(ErrorKind::Other, "boom"))
        }
        fn consume(&mut self, _: usize) {}
    }

    let count = parse_jsonl(AlwaysFails).take(5000).count();
    assert_eq!(count, 1, "reader must stop after the first I/O error");
}

/// Deeply nested JSON must be rejected with an error, never crash on a stack
/// overflow. Defense-in-depth for both the default `serde_json` path (which is
/// itself bounded at depth 128) and the `simd` path (which is not bounded by
/// `simd_json` and would otherwise overflow the stack).
#[test]
fn integration_deep_nesting_is_rejected() {
    let n = 5000usize;
    let line = format!("{}{}\n", "[".repeat(n), "]".repeat(n));
    let mut reader = parse_jsonl(BufReader::new(line.as_bytes()));
    let err = reader
        .next()
        .expect("must yield exactly one result")
        .unwrap_err();
    assert!(matches!(err.kind, JsonlErrorKind::Json(_)));
}
