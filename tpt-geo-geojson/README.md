# tpt-geo-geojson

[![docs.rs](https://docs.rs/tpt-geo-geojson/badge.svg)](https://docs.rs/tpt-geo-geojson)
[![crates.io](https://img.shields.io/crates/v/tpt-geo-geojson.svg)](https://crates.io/crates/tpt-geo-geojson)

Strict, validating GeoJSON parser with path-accurate error messages.

The existing `geojson` crate is good, but often lacks strict validation for malformed GIS data, which causes downstream mapping libraries to panic. This crate adds a second validation pass with precise error reporting.

## Features

- **Full GeoJSON support** — Point, MultiPoint, LineString, MultiLineString, Polygon, MultiPolygon, GeometryCollection, Feature, FeatureCollection
- **Strict validation** — coordinate arrays must have 2 or 3 elements; polygon rings must be closed with ≥ 4 positions
- **Path-accurate errors** — `GeoError.path` tells you exactly where in the structure the problem is (e.g. `features[2].geometry.coordinates[0]`)
- **Reader API** — parse from any `Read` source, not just strings

## Usage

```rust
use tpt_geo_geojson::parse;

let geojson = r#"{
    "type": "FeatureCollection",
    "features": [{
        "type": "Feature",
        "geometry": {"type": "Point", "coordinates": [125.6, 10.1]},
        "properties": {"name": "Dinagat Islands"}
    }]
}"#;

let result = parse(geojson).unwrap();
```

## Error handling

```rust
use tpt_geo_geojson::{parse, GeoErrorKind};

// 4-element coordinate array — invalid
let bad = r#"{"type":"Point","coordinates":[1,2,3,4]}"#;
let err = parse(bad).unwrap_err();
println!("path: {}", err.path);   // ".coordinates"
println!("error: {}", err.kind);  // "malformed coordinates: ..."
```

## Serialization & round-trip

Parsed values implement `Serialize`, so you can write valid GeoJSON back out with
[`to_json`](https://docs.rs/tpt-geo-geojson/latest/tpt_geo_geojson/fn.to_json.html).
`Feature` and `FeatureCollection` correctly emit their `"type"` member, and any
captured `bbox` or non-standard (foreign) members are preserved across a
parse → serialize → parse round-trip.

```rust
use tpt_geo_geojson::parse;

let geo = parse(r#"{"type":"FeatureCollection","bbox":[0,0,10,10],"features":[]}"#).unwrap();
let json = tpt_geo_geojson::to_json(&geo).unwrap();
assert!(json.contains(r#""type":"FeatureCollection""#));
assert!(json.contains("bbox"));

// Non-standard members survive the round-trip too:
let f = parse(r#"{"type":"Feature","properties":null,"title":"hi"}"#).unwrap();
let out = tpt_geo_geojson::to_json(&f).unwrap();
assert!(out.contains(r#""title":"hi""#));
```

## License

Licensed under either of [Apache License 2.0](../LICENSE-APACHE) or [MIT](../LICENSE-MIT) at your option.
