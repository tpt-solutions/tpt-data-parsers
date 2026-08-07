use criterion::{criterion_group, criterion_main, Criterion};
use tpt_mime_pure::detect;

fn bench_detect(c: &mut Criterion) {
    let buf = [
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01,
    ];
    c.bench_function("mime_detect", |b| {
        b.iter(|| {
            let _ = detect(&buf);
        });
    });
}

criterion_group!(benches, bench_detect);
criterion_main!(benches);
