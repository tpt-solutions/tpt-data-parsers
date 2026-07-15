//! Example: parse, then serialize back to valid GeoJSON (round-trip).
//!
//! Run with: `cargo run -p tpt-geo-geojson --example serialize`

use tpt_geo_geojson::parse;

fn main() {
    let input = r#"{"type":"FeatureCollection","bbox":[0,0,10,10],"features":[]}"#;
    let geo = parse(input).unwrap();
    let json = tpt_geo_geojson::to_json(&geo).unwrap();
    println!("{}", json);
    assert!(json.contains(r#""type":"FeatureCollection""#));
    assert!(json.contains("bbox"));
}
