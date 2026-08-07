use criterion::{criterion_group, criterion_main, Criterion};
use std::io::Cursor;
use tpt_jsonl_stream::parse_jsonl;

fn bench_parse(c: &mut Criterion) {
    let data = r#"{"a":1}
{"b":2}
{"c":3}
{"d":4}
{"e":5}"#;
    c.bench_function("jsonl_parse", |b| {
        b.iter(|| {
            let n = parse_jsonl(Cursor::new(data.as_bytes())).count();
            assert_eq!(n, 5);
        });
    });
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
