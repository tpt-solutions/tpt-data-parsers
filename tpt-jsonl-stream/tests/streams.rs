use std::fs::File;
use std::io::BufReader;
use tpt_jsonl_stream::parse_jsonl;

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
