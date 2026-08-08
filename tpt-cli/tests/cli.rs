//! Integration tests for the `tpt` CLI. These lock in the behaviour fixes from
//! the 2026-08-08 platform audit (exit codes, `--json` validity, timezone
//! semantics, deterministic `logfmt` ordering, MIME/extension fallback, sniff
//! precedence, and the broken-pipe-safe completions).
//!
//! Run with: `cargo test -p tpt-cli`
use std::io::Write;
use std::process::{Command, Stdio};

fn bin() -> Command {
    // The workspace builds a single binary named `tpt-cli`. Locate it in the
    // cargo target directory (this test executable lives next to it, or one
    // level down inside `deps`).
    let exe = std::env::current_exe().expect("current_exe");
    let mut dir = exe.parent().expect("exe parent").to_path_buf();
    let name = if cfg!(windows) {
        "tpt-cli.exe"
    } else {
        "tpt-cli"
    };
    for _ in 0..3 {
        let candidate = dir.join(name);
        if candidate.exists() {
            return Command::new(candidate);
        }
        dir = dir.parent().expect("walk up").to_path_buf();
    }
    panic!("could not find {name} near {}", exe.display());
}

fn run(args: &[&str]) -> (std::process::ExitStatus, String, String) {
    let out = bin().args(args).output().expect("failed to spawn tpt");
    (
        out.status,
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

fn write_tmp(name: &str, bytes: &[u8]) -> String {
    let dir = std::env::temp_dir().join("tpt-cli-tests");
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join(name);
    let mut f = std::fs::File::create(&p).unwrap();
    f.write_all(bytes).unwrap();
    p.to_string_lossy().to_string()
}

#[test]
fn cron_json_error_is_json() {
    let (st, _out, err) = run(&["cron", "99 * * * *", "--json"]);
    assert_eq!(st.code(), Some(1));
    let v: serde_json::Value = serde_json::from_str(err.trim()).expect("error JSON");
    assert_eq!(v["ok"], serde_json::Value::Bool(false));
    assert!(v["error"].is_string());
}

#[test]
fn cron_invalid_expr_exit_nonzero() {
    let (st, _out, _) = run(&["cron", "not a cron"]);
    assert_eq!(st.code(), Some(1));
}

#[test]
fn cron_human_json_fields_present() {
    let (st, out, _) = run(&["cron", "0 9 * * 1-5", "--json"]);
    assert!(st.success());
    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("json");
    assert_eq!(v["ok"], serde_json::Value::Bool(true));
    assert!(v["human"].is_string());
    assert!(v["expression"].is_string());
}

#[test]
fn cron_timezone_changes_schedule_not_just_display() {
    let (st, out, _) = run(&[
        "cron",
        "0 9 * * *",
        "--next",
        "--timezone",
        "+08:00",
        "--json",
    ]);
    assert!(st.success());
    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("json");
    let next = v["next"].as_str().expect("next field");
    assert!(
        next.ends_with("+08:00"),
        "expected +08:00 suffix, got {next}"
    );
    assert!(
        next.contains("09:00:00"),
        "expected 09:00 in +08:00, got {next}"
    );
}

#[test]
fn cron_six_field_next_is_utc_and_parseable() {
    let (st, out, _) = run(&[
        "cron",
        "*/10 * * * * *",
        "--next",
        "--timezone",
        "UTC",
        "--json",
    ]);
    assert!(st.success());
    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("json");
    let next = v["next"].as_str().expect("next field");
    assert!(next.ends_with("+00:00"), "expected UTC, got {next}");
}

#[test]
fn mime_extensionless_file_is_unknown() {
    let p = write_tmp("exe", b"");
    let (st, out, _) = run(&["mime", &p]);
    assert_eq!(st.code(), Some(1), "exit should be 1 for unknown type");
    assert!(out.trim().is_empty());
}

#[test]
fn mime_pdf_by_extension() {
    let p = write_tmp("report.pdf", b"%PDF-1.4 fake");
    let (st, out, _) = run(&["mime", &p]);
    assert!(st.success());
    assert_eq!(out.trim(), "application/pdf");
}

#[test]
fn mime_pdf_by_extension_only() {
    // Content-based detection now recognises JSON text, so a `.json` file whose
    // bytes are valid JSON is reported as `application/json` (exit 0).
    let p = write_tmp("data.json", b"{\"a\":1}");
    let (st, out, _) = run(&["mime", &p]);
    assert!(st.success());
    assert_eq!(out.trim(), "application/json");

    // A format with no magic bytes and an unrecognised extension is "unknown".
    // Use bytes that match no binary signature and are not valid UTF-8 text.
    let p1 = write_tmp("weird.xyz", &[0x01u8, 0x02, 0x03, 0x04, 0x05]);
    let (st1, out1, _) = run(&["mime", &p1]);
    assert_eq!(st1.code(), Some(1));
    assert!(out1.trim().is_empty());

    // A format the crate knows by both magic bytes and extension:
    let p2 = write_tmp("report.pdf", b"%PDF-1.4 fake");
    let (st2, out2, _) = run(&["mime", &p2]);
    assert!(st2.success());
    assert_eq!(out2.trim(), "application/pdf");
}

#[test]
fn mime_stdin_detection() {
    let mut child = bin()
        .args(["mime", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"\x00\x00\x01\x00icon")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert_eq!(s.trim(), "image/x-icon");
}

#[test]
fn logfmt_preserves_order_and_duplicates() {
    let (st, out, _) = run(&["logfmt", "b=2", "a=1", "b=3"]);
    assert!(st.success());
    let text = out.trim();
    let first = text.lines().next().unwrap();
    assert!(first.starts_with("b = 2"), "order not preserved: {text}");
    assert!(text.contains("b = 3"), "duplicate key dropped: {text}");
    assert_eq!(text.lines().count(), 3, "expected 3 fields, got {text}");
}

#[test]
fn logfmt_json_array_shape() {
    let (st, out, _) = run(&["logfmt", "k=v", "--json"]);
    assert!(st.success());
    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("json");
    let fields = v["fields"].as_array().expect("fields array");
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0]["key"], serde_json::Value::String("k".into()));
}

#[test]
fn sniff_runs_mime_detection_on_ascii_clean_tar() {
    let mut tar = vec![0u8; 600];
    tar[..12].copy_from_slice(b"notes.txt\0\0\0");
    tar[257..262].copy_from_slice(b"ustar");
    let p = write_tmp("clean.tar", &tar);
    let (st, out, _) = run(&["sniff", &p]);
    assert!(st.success());
    assert!(
        out.contains("application/x-tar"),
        "sniff should detect tar, got: {out}"
    );
}

#[test]
fn sniff_json_vs_geojson_distinguishable() {
    let geo = write_tmp("pt.json", br#"{"type":"Point","coordinates":[1,2]}"#);
    let (_, out_g, _) = run(&["sniff", &geo, "--json"]);
    let v_g: serde_json::Value = serde_json::from_str(out_g.trim()).expect("json");
    assert_eq!(v_g["results"][0]["format"], "geojson");

    let js = write_tmp("obj.json", br#"{"a":1}"#);
    let (_, out_j, _) = run(&["sniff", &js, "--json"]);
    let v_j: serde_json::Value = serde_json::from_str(out_j.trim()).expect("json");
    assert_eq!(v_j["results"][0]["format"], "json");
    assert!(v_j["results"][0]["format"] != "geojson");
}

#[test]
fn sniff_bare_scalars_not_jsonl() {
    let p = write_tmp("nums.txt", b"1\n2\n3\n");
    let (_, out, _) = run(&["sniff", &p]);
    assert!(
        !out.contains("jsonl"),
        "bare scalars should not be jsonl, got: {out}"
    );
}

#[test]
fn sniff_exits_nonzero_on_unknown_like_mime() {
    // `sniff` and `mime` must share the same exit-code convention: an
    // undetermined type is a failure (exit 1), not a success. Bytes that match no
    // magic signature and are not valid UTF-8 text classify as `unknown`.
    let p = write_tmp("mystery.xyz", &[0x01u8, 0x02, 0x03, 0x04, 0x05, 0x06]);
    let (st, out, _) = run(&["sniff", &p, "--json"]);
    assert_eq!(st.code(), Some(1));
    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("json");
    assert_eq!(v["ok"], serde_json::Value::Bool(false));
    assert_eq!(v["results"][0]["format"], "unknown");
}

#[test]
fn sniff_reports_truncated_for_large_files() {
    // A file larger than the sniff read window must still classify from its head
    // and flag that the input was truncated, rather than silently mis-reporting
    // it as `unknown`.
    let big = format!("{{\"x\":\"{}\"}}", "a".repeat(9_000_000)); // ~9 MB single JSON value
    let p = write_tmp("big.json", big.as_bytes());
    let (st, out, _) = run(&["sniff", &p, "--json"]);
    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("json");
    assert!(st.success(), "head is valid JSON, so sniff should succeed");
    let fmt = v["results"][0]["format"].as_str().unwrap();
    assert!(
        fmt == "json" || fmt == "application/json",
        "expected a JSON classification, got {fmt}"
    );
    assert_eq!(v["results"][0]["truncated"], serde_json::Value::Bool(true));
}

#[test]
fn geojson_validates() {
    let p = write_tmp("pt.geojson", br#"{"type":"Point","coordinates":[0,0]}"#);
    let (st, out, _) = run(&["geojson", &p]);
    assert!(st.success(), "geojson should validate");
    assert!(out.trim().is_empty() || out.trim() == "valid");
}

#[test]
fn jsonl_counts_records() {
    let p = write_tmp("data.jsonl", b"{\"a\":1}\n{\"b\":2}\n");
    let (st, out, _) = run(&["jsonl", &p]);
    assert!(st.success());
    assert!(out.contains("records: 2"), "got: {out}");
}

#[test]
fn completions_produces_script() {
    let (st, out, _) = run(&["completions", "bash"]);
    assert!(st.success());
    assert!(out.contains("tpt"), "completion script missing marker");
}
