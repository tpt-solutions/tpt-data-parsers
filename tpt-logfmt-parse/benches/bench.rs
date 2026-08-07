use criterion::{criterion_group, criterion_main, Criterion};
use tpt_logfmt_parse::LogfmtParser;

fn bench_parse(c: &mut Criterion) {
    let input = "level=info msg=\"hello world\" latency=42ms path=/a/b/c retries=3";
    c.bench_function("logfmt_parse", |b| {
        b.iter(|| {
            let _: Vec<_> = LogfmtParser::new(input)
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
        });
    });
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
