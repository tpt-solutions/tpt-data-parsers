//! Property-based round-trip tests for the GeoJSON parse/serialize cycle.

use proptest::prelude::*;
use std::collections::BTreeMap;
use tpt_geo_geojson::{parse, to_json, Feature, GeoJson, Geometry, Position};

fn arb_position() -> impl Strategy<Value = Position> {
    (
        -180..=180_i64,
        -90..=90_i64,
        proptest::option::of(-1000..=1000_i64),
    )
        .prop_map(|(lon, lat, alt)| {
            let mut coords = vec![lon as f64, lat as f64];
            if let Some(a) = alt {
                coords.push(a as f64);
            }
            Position::new(coords).unwrap()
        })
}

fn arb_ring() -> impl Strategy<Value = Vec<Position>> {
    proptest::collection::vec(arb_position(), 3..6).prop_map(|mut pts| {
        // Close the ring (first == last) so it satisfies RFC 7946.
        pts.push(pts[0].clone());
        pts
    })
}

fn arb_geometry() -> impl Strategy<Value = Geometry> {
    prop_oneof![
        arb_position().prop_map(|p| Geometry::Point { coordinates: p }),
        proptest::collection::vec(arb_position(), 2..6)
            .prop_map(|pts| Geometry::LineString { coordinates: pts }),
        arb_ring().prop_map(|ring| Geometry::Polygon { coordinates: vec![ring] }),
    ]
}

proptest! {
    /// A bare geometry round-trips through `to_json` -> `parse` exactly.
    #[test]
    fn geometry_round_trips(geom in arb_geometry()) {
        let geo = GeoJson::Geometry(geom);
        let json = to_json(&geo).unwrap();
        let back = parse(&json).unwrap();
        prop_assert_eq!(back, geo);
    }

    /// A feature carrying a geometry (and no foreign members) round-trips.
    #[test]
    fn feature_round_trips(geom in arb_geometry()) {
        let feature = Feature {
            geometry: Some(geom),
            properties: None,
            id: None,
            bbox: None,
            foreign_members: BTreeMap::new(),
        };
        let geo = GeoJson::Feature(feature);
        let json = to_json(&geo).unwrap();
        let back = parse(&json).unwrap();
        prop_assert_eq!(back, geo);
    }
}
