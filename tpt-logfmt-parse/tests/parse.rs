use std::io::BufReader;
use tpt_logfmt_parse::{
    format_pair, parse_logfmt_lines, parse_to_map, parse_to_pairs, write_logfmt, LogfmtLinesReader,
    LogfmtParser,
};

fn pair(key: &str, value: &str) -> (String, String) {
    (key.to_string(), value.to_string())
}

#[test]
fn integration_heroku_log_line() {
    let line =
        r#"at=info method=GET path="/api/v1/users" host=example.com status=200 duration=12ms"#;
    let map = parse_to_map(line).unwrap();
    assert_eq!(map["at"], "info");
    assert_eq!(map["method"], "GET");
    assert_eq!(map["path"], "/api/v1/users");
    assert_eq!(map["status"], "200");
    assert_eq!(map["duration"], "12ms");
}

#[test]
fn integration_zero_copy_iterator_collects_all() {
    let line = "ts=2024-01-01T00:00:00Z level=warn caller=main.go:42 msg=timeout";
    let pairs: Vec<_> = LogfmtParser::new(line).collect::<Result<_, _>>().unwrap();
    assert_eq!(pairs.len(), 4);
    assert_eq!(pairs[0].0, "ts");
    assert_eq!(pairs[0].1, "2024-01-01T00:00:00Z");
    assert_eq!(pairs[1].1, "warn");
    assert_eq!(pairs[2].1, "main.go:42");
    assert_eq!(pairs[3].1, "timeout");
}

#[test]
fn integration_multiple_bare_keys() {
    let line = "debug verbose quiet level=info";
    let pairs = parse_to_pairs(line).unwrap();
    assert_eq!(
        pairs,
        vec![
            pair("debug", ""),
            pair("verbose", ""),
            pair("quiet", ""),
            pair("level", "info"),
        ]
    );
}

#[test]
fn integration_empty_quoted_value() {
    let line = r#"key="" other=val"#;
    let map = parse_to_map(line).unwrap();
    assert_eq!(map["key"], "");
    assert_eq!(map["other"], "val");
}

#[test]
fn integration_escape_newline_tab() {
    let line = r#"msg="line1\nline2\ttabbed""#;
    let map = parse_to_map(line).unwrap();
    assert_eq!(map["msg"], "line1\nline2\ttabbed");
}

#[test]
fn integration_iterator_never_stalls_on_error() {
    let line = r#"=orphan "quoted"=1 ok=yes"#;
    let mut seen_ok = false;
    let mut steps = 0;
    for item in LogfmtParser::new(line) {
        steps += 1;
        assert!(steps < 32, "iterator failed to make progress");
        if let Ok((key, value)) = item {
            if key == "ok" {
                assert_eq!(value, "yes");
                seen_ok = true;
            }
        }
    }
    assert!(seen_ok);
}

#[test]
fn integration_crlf_and_tabs_are_separators() {
    assert_eq!(parse_to_map("a=1\r\n").unwrap()["a"], "1");
    assert_eq!(
        parse_to_pairs("a=1\tb=2\r\nc=3\n").unwrap(),
        vec![pair("a", "1"), pair("b", "2"), pair("c", "3")]
    );
}

#[test]
fn integration_parser_and_map_agree() {
    let line = r#"msg="say \"hi\"" path="a\\b" plain=1"#;
    let map = parse_to_map(line).unwrap();
    for (key, value) in parse_to_pairs(line).unwrap() {
        assert_eq!(map[&key], value);
    }
    assert_eq!(map["msg"], r#"say "hi""#);
    assert_eq!(map["path"], r"a\b");
}

#[test]
fn integration_writer_round_trips() {
    let pairs = vec![
        pair("level", "info"),
        pair("odd key", "value with spaces"),
        pair("quote", r#"say "hi""#),
        pair("slash", r"a\b"),
        pair("ctrl", "a\nb\tc\rd\u{1}"),
        pair("empty", ""),
    ];
    let line = write_logfmt(pairs.clone());
    assert!(!line.contains('\n'));
    assert_eq!(parse_to_pairs(&line).unwrap(), pairs);
    assert_eq!(format_pair("odd key", "x"), r#""odd key"=x"#);
}

#[test]
fn integration_lines_reader_heroku_stream() {
    let data = b"at=info method=GET status=200\n\nat=error method=POST status=503 msg=\"boom \\\"now\\\"\"\n";
    let lines: Vec<_> = LogfmtLinesReader::new(BufReader::new(data.as_slice()))
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0][0], pair("at", "info"));
    assert_eq!(lines[1][3], pair("msg", r#"boom "now""#));
}

#[test]
fn integration_lines_reader_reports_bad_line() {
    let data = b"a=1\nb=\"unterminated\nc=3\n";
    let mut reader = parse_logfmt_lines(BufReader::new(data.as_slice()));
    assert!(reader.next().unwrap().is_ok());
    let err = reader.next().unwrap().unwrap_err();
    assert_eq!(err.line, 2);
    assert!(reader.next().unwrap().is_ok());
}
