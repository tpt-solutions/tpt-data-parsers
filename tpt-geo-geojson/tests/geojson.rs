use std::fs;
use tpt_geo_geojson::{parse, GeoErrorKind, GeoJson, Geometry};

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name);
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("fixture '{}' not found", name))
}

#[test]
fn integration_valid_feature_collection() {
    let s = fixture("valid.geojson");
    let geo = parse(&s).unwrap();
    match geo {
        GeoJson::FeatureCollection(fc) => {
            assert_eq!(fc.features.len(), 3);
            assert!(fc.features[0].geometry.is_some());
        }
        other => panic!("expected FeatureCollection, got {:?}", other),
    }
}

#[test]
fn integration_malformed_coords_error() {
    let s = fixture("malformed_coords.geojson");
    let err = parse(&s).unwrap_err();
    assert!(matches!(err.kind, GeoErrorKind::MalformedCoordinates(_)));
    assert!(!err.path.is_empty());
}

#[test]
fn integration_unclosed_polygon_error() {
    let s = fixture("unclosed_polygon.geojson");
    let err = parse(&s).unwrap_err();
    assert!(matches!(err.kind, GeoErrorKind::InvalidRing(_)));
    assert!(!err.to_string().is_empty());
}

#[test]
fn integration_point_coordinates() {
    let geo = parse(r#"{"type":"Point","coordinates":[125.6,10.1]}"#).unwrap();
    if let GeoJson::Geometry(Geometry::Point { coordinates: p }) = geo {
        assert!((p.longitude() - 125.6).abs() < 1e-9);
        assert!((p.latitude() - 10.1).abs() < 1e-9);
    } else {
        panic!("expected Point");
    }
}

#[test]
fn integration_multipolygon() {
    let s = r#"{"type":"MultiPolygon","coordinates":[[[[0,0],[1,0],[1,1],[0,1],[0,0]]],[[[2,2],[3,2],[3,3],[2,3],[2,2]]]]}"#;
    let geo = parse(s).unwrap();
    assert!(matches!(
        geo,
        GeoJson::Geometry(Geometry::MultiPolygon { .. })
    ));
}

#[test]
fn integration_feature_null_geometry() {
    let s = r#"{"type":"Feature","geometry":null,"properties":null}"#;
    let geo = parse(s).unwrap();
    if let GeoJson::Feature(f) = geo {
        assert!(f.geometry.is_none());
    } else {
        panic!();
    }
}
