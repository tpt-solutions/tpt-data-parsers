use criterion::{criterion_group, criterion_main, Criterion};
use tpt_cron_parse::CronExpr;

fn bench_parse(c: &mut Criterion) {
    c.bench_function("cron_parse_and_describe", |b| {
        b.iter(|| {
            let expr = CronExpr::parse("*/15 0 9 * * 1-5").unwrap();
            let _ = expr.to_human_readable();
        });
    });
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
