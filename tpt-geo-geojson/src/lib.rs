#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use serde::Serialize;
use serde_json::Value;
use std::fmt;
use std::io::Read;

// ---- Error types ----

/// The kind of validation or parse error.
#[derive(Debug)]
pub enum GeoErrorKind {
    /// The GeoJSON `type` field has an unexpected value.
    InvalidType(String),
    /// A coordinate array has the wrong length or structure.
    MalformedCoordinates(String),
    /// A polygon ring is not closed or has fewer than 4 positions.
    InvalidRing(String),
    /// The optional `crs` field is malformed.
    InvalidCrs(String),
    /// An I/O error reading the input.
    Io(std::io::Error),
    /// A JSON deserialization error.
    Json(serde_json::Error),
}

impl fmt::Display for GeoErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidType(s) => write!(f, "invalid type: {}", s),
            Self::MalformedCoordinates(s) => write!(f, "malformed coordinates: {}", s),
            Self::InvalidRing(s) => write!(f, "invalid ring: {}", s),
            Self::InvalidCrs(s) => write!(f, "invalid CRS: {}", s),
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::Json(e) => write!(f, "JSON error: {}", e),
        }
    }
}

/// A GeoJSON parse or validation error with a path into the structure.
///
/// The `path` field uses dot/bracket notation to locate the error, e.g.
/// `"features[2].geometry.coordinates[0]"`.
#[derive(Debug)]
pub struct GeoError {
    /// The kind of error.
    pub kind: GeoErrorKind,
    /// A dot/bracket path into the GeoJSON structure where the error occurred.
    pub path: String,
}

impl fmt::Display for GeoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "geojson error at {}: {}", self.path, self.kind)
    }
}

impl std::error::Error for GeoError {}

// ---- Coordinate types ----

/// A GeoJSON position: `[longitude, latitude]` or `[longitude, latitude, altitude]`.
///
/// GeoJSON (RFC 7946) requires at least two elements; a third optional element is altitude.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Position(pub Vec<f64>);

impl Position {
    /// Longitude (first element).
    pub fn longitude(&self) -> f64 {
        self.0[0]
    }
    /// Latitude (second element).
    pub fn latitude(&self) -> f64 {
        self.0[1]
    }
    /// Altitude, if present (third element).
    pub fn altitude(&self) -> Option<f64> {
        self.0.get(2).copied()
    }
}

// ---- Geometry types ----

/// A GeoJSON geometry object.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum Geometry {
    /// A single point.
    Point {
        /// The point's position.
        coordinates: Position,
    },
    /// Multiple points.
    MultiPoint {
        /// The positions of each point.
        coordinates: Vec<Position>,
    },
    /// A line string.
    LineString {
        /// The ordered sequence of positions forming the line.
        coordinates: Vec<Position>,
    },
    /// Multiple line strings.
    MultiLineString {
        /// The ordered sequences of positions for each line.
        coordinates: Vec<Vec<Position>>,
    },
    /// A polygon (first ring is exterior, remaining rings are holes).
    Polygon {
        /// Rings: first is the exterior boundary, rest are holes.
        coordinates: Vec<Vec<Position>>,
    },
    /// Multiple polygons.
    MultiPolygon {
        /// Each element is a polygon's ring array.
        coordinates: Vec<Vec<Vec<Position>>>,
    },
    /// A collection of heterogeneous geometries.
    GeometryCollection {
        /// The geometries in this collection.
        geometries: Vec<Geometry>,
    },
}

// ---- Feature types ----

/// A GeoJSON Feature.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Feature {
    /// The feature's geometry, if any.
    pub geometry: Option<Geometry>,
    /// Arbitrary properties associated with the feature.
    pub properties: Option<Value>,
    /// An optional feature identifier.
    pub id: Option<Value>,
}

/// A GeoJSON FeatureCollection.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FeatureCollection {
    /// The features in this collection.
    pub features: Vec<Feature>,
}

/// The top-level GeoJSON object.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum GeoJson {
    /// A single Feature.
    Feature(Feature),
    /// A collection of Features.
    FeatureCollection(FeatureCollection),
    /// A bare Geometry.
    Geometry(Geometry),
}

// ---- Public API ----

/// Parse a GeoJSON string.
///
/// Performs a full validation pass after deserialization:
/// - Coordinate arrays must have 2 or 3 elements.
/// - Polygon rings must be closed (first == last) and have ≥ 4 positions.
///
/// # Example
///
/// ```
/// use tpt_geo_geojson::parse;
///
/// let geojson = r#"{"type":"Point","coordinates":[125.6,10.1]}"#;
/// let result = parse(geojson).unwrap();
/// ```
pub fn parse(input: &str) -> Result<GeoJson, GeoError> {
    let raw: Value = serde_json::from_str(input).map_err(|e| GeoError {
        kind: GeoErrorKind::Json(e),
        path: String::new(),
    })?;
    parse_value(&raw, "")
}

/// Parse GeoJSON from any [`Read`] source.
pub fn parse_reader<R: Read>(mut reader: R) -> Result<GeoJson, GeoError> {
    let raw: Value = serde_json::from_reader(&mut reader).map_err(|e| GeoError {
        kind: GeoErrorKind::Json(e),
        path: String::new(),
    })?;
    parse_value(&raw, "")
}

// ---- Internal parsing ----

fn parse_value(v: &Value, path: &str) -> Result<GeoJson, GeoError> {
    let type_str = v
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| GeoError {
            kind: GeoErrorKind::InvalidType("missing or non-string 'type' field".into()),
            path: format!("{}.type", path),
        })?;

    match type_str {
        "FeatureCollection" => {
            let features_val =
                v.get("features")
                    .and_then(Value::as_array)
                    .ok_or_else(|| GeoError {
                        kind: GeoErrorKind::InvalidType(
                            "FeatureCollection missing 'features' array".into(),
                        ),
                        path: format!("{}.features", path),
                    })?;
            let mut features = Vec::with_capacity(features_val.len());
            for (i, fv) in features_val.iter().enumerate() {
                let prefix = if path.is_empty() {
                    String::new()
                } else {
                    format!("{}.", path)
                };
                let fp = format!("{}features[{}]", prefix, i);
                features.push(parse_feature(fv, &fp)?);
            }
            Ok(GeoJson::FeatureCollection(FeatureCollection { features }))
        }
        "Feature" => {
            let fp = if path.is_empty() {
                String::new()
            } else {
                path.to_owned()
            };
            Ok(GeoJson::Feature(parse_feature(v, &fp)?))
        }
        _ => {
            let geom = parse_geometry(v, path)?;
            Ok(GeoJson::Geometry(geom))
        }
    }
}

fn parse_feature(v: &Value, path: &str) -> Result<Feature, GeoError> {
    if v.get("type").and_then(Value::as_str) != Some("Feature") {
        return Err(GeoError {
            kind: GeoErrorKind::InvalidType(format!("expected 'Feature', got {:?}", v.get("type"))),
            path: format!("{}.type", path),
        });
    }

    let geometry = match v.get("geometry") {
        None | Some(Value::Null) => None,
        Some(geom_val) => {
            let gp = format!("{}.geometry", path);
            Some(parse_geometry(geom_val, &gp)?)
        }
    };

    let properties = v.get("properties").cloned();
    let id = v.get("id").cloned();

    Ok(Feature {
        geometry,
        properties,
        id,
    })
}

fn parse_geometry(v: &Value, path: &str) -> Result<Geometry, GeoError> {
    let type_str = v
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| GeoError {
            kind: GeoErrorKind::InvalidType("geometry missing 'type' field".into()),
            path: format!("{}.type", path),
        })?;

    match type_str {
        "Point" => {
            let coords_path = format!("{}.coordinates", path);
            let raw = coords_raw(v, &coords_path)?;
            let pos = parse_position(raw, &coords_path)?;
            Ok(Geometry::Point { coordinates: pos })
        }
        "MultiPoint" => {
            let coords_path = format!("{}.coordinates", path);
            let arr = coords_raw(v, &coords_path)?;
            let positions = parse_position_array(arr, &coords_path)?;
            Ok(Geometry::MultiPoint {
                coordinates: positions,
            })
        }
        "LineString" => {
            let coords_path = format!("{}.coordinates", path);
            let arr = coords_raw(v, &coords_path)?;
            let positions = parse_position_array(arr, &coords_path)?;
            Ok(Geometry::LineString {
                coordinates: positions,
            })
        }
        "MultiLineString" => {
            let coords_path = format!("{}.coordinates", path);
            let arr = coords_raw(v, &coords_path)?
                .as_array()
                .ok_or_else(|| GeoError {
                    kind: GeoErrorKind::MalformedCoordinates(
                        "expected array of line strings".into(),
                    ),
                    path: coords_path.clone(),
                })?;
            let mut lines = Vec::with_capacity(arr.len());
            for (i, line_val) in arr.iter().enumerate() {
                let lp = format!("{}[{}]", coords_path, i);
                let line_arr = line_val.as_array().ok_or_else(|| GeoError {
                    kind: GeoErrorKind::MalformedCoordinates("expected array".into()),
                    path: lp.clone(),
                })?;
                lines.push(parse_position_array(line_val, &lp)?);
                let _ = line_arr;
            }
            Ok(Geometry::MultiLineString { coordinates: lines })
        }
        "Polygon" => {
            let coords_path = format!("{}.coordinates", path);
            let rings = parse_rings(v, &coords_path)?;
            Ok(Geometry::Polygon { coordinates: rings })
        }
        "MultiPolygon" => {
            let coords_path = format!("{}.coordinates", path);
            let arr = coords_raw(v, &coords_path)?
                .as_array()
                .ok_or_else(|| GeoError {
                    kind: GeoErrorKind::MalformedCoordinates("expected array of polygons".into()),
                    path: coords_path.clone(),
                })?;
            let mut polys = Vec::with_capacity(arr.len());
            for (i, poly_val) in arr.iter().enumerate() {
                let pp = format!("{}[{}]", coords_path, i);
                let rings = parse_rings_value(poly_val, &pp)?;
                polys.push(rings);
            }
            Ok(Geometry::MultiPolygon { coordinates: polys })
        }
        "GeometryCollection" => {
            let geoms_val = v
                .get("geometries")
                .and_then(Value::as_array)
                .ok_or_else(|| GeoError {
                    kind: GeoErrorKind::InvalidType(
                        "GeometryCollection missing 'geometries' array".into(),
                    ),
                    path: format!("{}.geometries", path),
                })?;
            let mut geoms = Vec::with_capacity(geoms_val.len());
            for (i, gv) in geoms_val.iter().enumerate() {
                let gp = format!("{}.geometries[{}]", path, i);
                geoms.push(parse_geometry(gv, &gp)?);
            }
            Ok(Geometry::GeometryCollection { geometries: geoms })
        }
        other => Err(GeoError {
            kind: GeoErrorKind::InvalidType(format!("unknown geometry type '{}'", other)),
            path: format!("{}.type", path),
        }),
    }
}

fn coords_raw<'a>(v: &'a Value, path: &str) -> Result<&'a Value, GeoError> {
    v.get("coordinates").ok_or_else(|| GeoError {
        kind: GeoErrorKind::MalformedCoordinates("missing 'coordinates' field".into()),
        path: path.to_owned(),
    })
}

fn parse_position(v: &Value, path: &str) -> Result<Position, GeoError> {
    let arr = v.as_array().ok_or_else(|| GeoError {
        kind: GeoErrorKind::MalformedCoordinates("position must be an array".into()),
        path: path.to_owned(),
    })?;
    if arr.len() < 2 || arr.len() > 3 {
        return Err(GeoError {
            kind: GeoErrorKind::MalformedCoordinates(format!(
                "position must have 2 or 3 elements, got {}",
                arr.len()
            )),
            path: path.to_owned(),
        });
    }
    let coords: Result<Vec<f64>, _> = arr
        .iter()
        .map(|n| {
            n.as_f64().ok_or_else(|| GeoError {
                kind: GeoErrorKind::MalformedCoordinates("coordinate must be a number".into()),
                path: path.to_owned(),
            })
        })
        .collect();
    Ok(Position(coords?))
}

fn parse_position_array(v: &Value, path: &str) -> Result<Vec<Position>, GeoError> {
    let arr = v.as_array().ok_or_else(|| GeoError {
        kind: GeoErrorKind::MalformedCoordinates("expected array of positions".into()),
        path: path.to_owned(),
    })?;
    let mut positions = Vec::with_capacity(arr.len());
    for (i, pv) in arr.iter().enumerate() {
        let pp = format!("{}[{}]", path, i);
        positions.push(parse_position(pv, &pp)?);
    }
    Ok(positions)
}

fn parse_rings(v: &Value, path: &str) -> Result<Vec<Vec<Position>>, GeoError> {
    let arr = coords_raw(v, path)?.as_array().ok_or_else(|| GeoError {
        kind: GeoErrorKind::MalformedCoordinates(
            "polygon coordinates must be an array of rings".into(),
        ),
        path: path.to_owned(),
    })?;
    parse_ring_array(arr, path)
}

fn parse_rings_value(v: &Value, path: &str) -> Result<Vec<Vec<Position>>, GeoError> {
    let arr = v.as_array().ok_or_else(|| GeoError {
        kind: GeoErrorKind::MalformedCoordinates("polygon must be an array of rings".into()),
        path: path.to_owned(),
    })?;
    parse_ring_array(arr, path)
}

fn parse_ring_array(arr: &[Value], path: &str) -> Result<Vec<Vec<Position>>, GeoError> {
    let mut rings = Vec::with_capacity(arr.len());
    for (i, ring_val) in arr.iter().enumerate() {
        let rp = format!("{}[{}]", path, i);
        let positions = parse_position_array(ring_val, &rp)?;
        // RFC 7946: ring must have ≥ 4 positions and be closed
        if positions.len() < 4 {
            return Err(GeoError {
                kind: GeoErrorKind::InvalidRing(format!(
                    "ring must have at least 4 positions, got {}",
                    positions.len()
                )),
                path: rp,
            });
        }
        let first = &positions[0];
        let last = &positions[positions.len() - 1];
        if (first.longitude() - last.longitude()).abs() > f64::EPSILON
            || (first.latitude() - last.latitude()).abs() > f64::EPSILON
        {
            return Err(GeoError {
                kind: GeoErrorKind::InvalidRing(
                    "polygon ring is not closed (first position != last position)".into(),
                ),
                path: rp,
            });
        }
        rings.push(positions);
    }
    Ok(rings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_point() {
        let s = r#"{"type":"Point","coordinates":[125.6,10.1]}"#;
        let geo = parse(s).unwrap();
        assert!(matches!(geo, GeoJson::Geometry(Geometry::Point { .. })));
    }

    #[test]
    fn parse_point_with_altitude() {
        let s = r#"{"type":"Point","coordinates":[0.0,0.0,100.0]}"#;
        let geo = parse(s).unwrap();
        if let GeoJson::Geometry(Geometry::Point { coordinates: p }) = geo {
            assert_eq!(p.altitude(), Some(100.0));
        } else {
            panic!("expected point");
        }
    }

    #[test]
    fn parse_feature() {
        let s = r#"{"type":"Feature","geometry":{"type":"Point","coordinates":[0,0]},"properties":{"name":"test"}}"#;
        let geo = parse(s).unwrap();
        assert!(matches!(geo, GeoJson::Feature(_)));
    }

    #[test]
    fn parse_feature_collection() {
        let s = r#"{
            "type":"FeatureCollection",
            "features":[
                {"type":"Feature","geometry":{"type":"Point","coordinates":[1,2]},"properties":null}
            ]
        }"#;
        let geo = parse(s).unwrap();
        if let GeoJson::FeatureCollection(fc) = geo {
            assert_eq!(fc.features.len(), 1);
        } else {
            panic!();
        }
    }

    #[test]
    fn parse_valid_polygon() {
        let s = r#"{"type":"Polygon","coordinates":[[[0,0],[1,0],[1,1],[0,1],[0,0]]]}"#;
        let geo = parse(s).unwrap();
        assert!(matches!(geo, GeoJson::Geometry(Geometry::Polygon { .. })));
    }

    #[test]
    fn error_on_wrong_coord_length() {
        let s = r#"{"type":"Point","coordinates":[1,2,3,4]}"#;
        let err = parse(s).unwrap_err();
        assert!(matches!(err.kind, GeoErrorKind::MalformedCoordinates(_)));
        assert!(err.path.contains("coordinates"));
    }

    #[test]
    fn error_on_unclosed_polygon() {
        let s = r#"{"type":"Polygon","coordinates":[[[0,0],[1,0],[1,1],[0,1]]]}"#;
        let err = parse(s).unwrap_err();
        assert!(matches!(err.kind, GeoErrorKind::InvalidRing(_)));
    }

    #[test]
    fn error_on_short_ring() {
        let s = r#"{"type":"Polygon","coordinates":[[[0,0],[1,0],[0,0]]]}"#;
        let err = parse(s).unwrap_err();
        assert!(matches!(err.kind, GeoErrorKind::InvalidRing(_)));
    }

    #[test]
    fn error_on_missing_type() {
        let s = r#"{"coordinates":[0,0]}"#;
        let err = parse(s).unwrap_err();
        assert!(matches!(err.kind, GeoErrorKind::InvalidType(_)));
    }

    #[test]
    fn error_has_nonempty_path_for_nested() {
        let s = r#"{
            "type":"FeatureCollection",
            "features":[
                {"type":"Feature","geometry":{"type":"Point","coordinates":[1,2,3,4]},"properties":null}
            ]
        }"#;
        let err = parse(s).unwrap_err();
        assert!(
            !err.path.is_empty(),
            "path should be non-empty: {:?}",
            err.path
        );
    }

    #[test]
    fn parse_linestring() {
        let s = r#"{"type":"LineString","coordinates":[[0,0],[1,1],[2,2]]}"#;
        assert!(matches!(
            parse(s).unwrap(),
            GeoJson::Geometry(Geometry::LineString { .. })
        ));
    }

    #[test]
    fn parse_geometry_collection() {
        let s =
            r#"{"type":"GeometryCollection","geometries":[{"type":"Point","coordinates":[0,0]}]}"#;
        assert!(matches!(
            parse(s).unwrap(),
            GeoJson::Geometry(Geometry::GeometryCollection { .. })
        ));
    }

    #[test]
    fn parse_reader_api() {
        let data = br#"{"type":"Point","coordinates":[1.0,2.0]}"#;
        let geo = parse_reader(data.as_slice()).unwrap();
        assert!(matches!(geo, GeoJson::Geometry(Geometry::Point { .. })));
    }
}
