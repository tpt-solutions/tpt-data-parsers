use tpt_logfmt_parse::{parse_to_map, LogfmtParser};

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
    assert_eq!(pairs[0], ("ts", "2024-01-01T00:00:00Z"));
    assert_eq!(pairs[1], ("level", "warn"));
    assert_eq!(pairs[2], ("caller", "main.go:42"));
    assert_eq!(pairs[3], ("msg", "timeout"));
}

#[test]
fn integration_multiple_bare_keys() {
    let line = "debug verbose quiet level=info";
    let pairs: Vec<_> = LogfmtParser::new(line).collect::<Result<_, _>>().unwrap();
    assert_eq!(pairs[0], ("debug", ""));
    assert_eq!(pairs[1], ("verbose", ""));
    assert_eq!(pairs[2], ("quiet", ""));
    assert_eq!(pairs[3], ("level", "info"));
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
