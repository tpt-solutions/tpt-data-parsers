//! Example: validate and parse a GeoJSON document.
//!
//! Run with: `cargo run -p tpt-geo-geojson --example validate`

use tpt_geo_geojson::parse;

fn main() {
    let input = r#"{
        "type": "FeatureCollection",
        "features": [
            {"type": "Feature", "geometry": {"type": "Point", "coordinates": [125.6, 10.1]}, "properties": null}
        ]
    }"#;

    match parse(input) {
        Ok(geo) => println!("parsed OK: {:?}", geo),
        Err(e) => eprintln!("error at {}: {}", e.path, e.kind),
    }

    // A malformed polygon (unclosed ring) demonstrates the validation pass.
    let bad = r#"{"type":"Polygon","coordinates":[[[0,0],[1,0],[1,1],[0,1]]]}"#;
    if let Err(e) = parse(bad) {
        eprintln!("expected validation error: {} ({})", e.path, e.kind);
    }
}
