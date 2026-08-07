use criterion::{criterion_group, criterion_main, Criterion};
use tpt_geo_geojson::parse;

fn bench_parse(c: &mut Criterion) {
    let geo = r#"{"type":"FeatureCollection","features":[{"type":"Feature","geometry":{"type":"Point","coordinates":[125.6,10.1]},"properties":{}}]}"#;
    c.bench_function("geo_parse", |b| {
        b.iter(|| {
            let _ = parse(geo).unwrap();
        });
    });
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
