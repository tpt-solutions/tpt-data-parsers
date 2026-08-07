#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use serde::ser::SerializeMap;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
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
/// Construct via [`Position::new`] which validates the length, so [`Position::longitude`]
/// and [`Position::latitude`] can never panic on a too-short vector.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Position(Vec<f64>);

impl Position {
    /// Construct a position, validating that `coords` has 2 or 3 elements.
    ///
    /// Returns a [`GeoErrorKind::MalformedCoordinates`] error otherwise.
    pub fn new(coords: Vec<f64>) -> Result<Position, GeoError> {
        if coords.len() < 2 || coords.len() > 3 {
            return Err(GeoError {
                kind: GeoErrorKind::MalformedCoordinates(format!(
                    "position must have 2 or 3 elements, got {}",
                    coords.len()
                )),
                path: String::new(),
            });
        }
        Ok(Position(coords))
    }

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

/// An axis-aligned bounding box computed from a geometry's own positions.
///
/// Longitude/latitude bounds are always present; altitude bounds are present only
/// when at least one position carries a third coordinate.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BoundingBox {
    /// Minimum longitude.
    pub west: f64,
    /// Minimum latitude.
    pub south: f64,
    /// Maximum longitude.
    pub east: f64,
    /// Maximum latitude.
    pub north: f64,
    /// Minimum altitude, if any position has one.
    pub min_altitude: Option<f64>,
    /// Maximum altitude, if any position has one.
    pub max_altitude: Option<f64>,
}

impl BoundingBox {
    /// Convert to the RFC 7946 `bbox` array order.
    ///
    /// Returns `[west, south, east, north]`, or
    /// `[west, south, min_altitude, east, north, max_altitude]` when altitude bounds
    /// are present.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_geo_geojson::{parse, GeoJson};
    ///
    /// let geo = parse(r#"{"type":"LineString","coordinates":[[0,0],[2,3]]}"#).unwrap();
    /// let GeoJson::Geometry(geom) = geo else { panic!("expected geometry") };
    /// assert_eq!(geom.bounding_box().unwrap().to_vec(), vec![0.0, 0.0, 2.0, 3.0]);
    /// ```
    pub fn to_vec(&self) -> Vec<f64> {
        match (self.min_altitude, self.max_altitude) {
            (Some(min), Some(max)) => vec![self.west, self.south, min, self.east, self.north, max],
            _ => vec![self.west, self.south, self.east, self.north],
        }
    }
}

impl Geometry {
    /// Re-run the strict validation pass on a (possibly hand-constructed) geometry.
    ///
    /// This runs exactly the same checks [`parse`] applies: positions must have 2 or 3
    /// finite coordinates within longitude ∈ [-180, 180] and latitude ∈ [-90, 90], and
    /// polygon rings must have at least 4 positions and be exactly closed.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_geo_geojson::{Geometry, Position};
    ///
    /// let ok = Geometry::Point {
    ///     coordinates: Position::new(vec![1.0, 2.0]).unwrap(),
    /// };
    /// assert!(ok.validate().is_ok());
    /// ```
    pub fn validate(&self) -> Result<(), GeoError> {
        validate_geometry_at(self, "")
    }

    /// Compute the bounding box of this geometry from its own positions.
    ///
    /// Returns `None` when the geometry contains no positions at all (for example an
    /// empty `MultiPoint` or an empty `GeometryCollection`).
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_geo_geojson::{parse, GeoJson};
    ///
    /// let geo = parse(r#"{"type":"MultiPoint","coordinates":[[0,0],[4,5]]}"#).unwrap();
    /// let GeoJson::Geometry(geom) = geo else { panic!("expected geometry") };
    /// let bbox = geom.bounding_box().unwrap();
    /// assert_eq!((bbox.west, bbox.south, bbox.east, bbox.north), (0.0, 0.0, 4.0, 5.0));
    /// ```
    pub fn bounding_box(&self) -> Option<BoundingBox> {
        let mut west = f64::INFINITY;
        let mut south = f64::INFINITY;
        let mut east = f64::NEG_INFINITY;
        let mut north = f64::NEG_INFINITY;
        let mut min_altitude: Option<f64> = None;
        let mut max_altitude: Option<f64> = None;
        let mut seen = false;
        self.for_each_position(&mut |p| {
            seen = true;
            west = west.min(p.longitude());
            east = east.max(p.longitude());
            south = south.min(p.latitude());
            north = north.max(p.latitude());
            if let Some(alt) = p.altitude() {
                min_altitude = Some(min_altitude.map_or(alt, |m: f64| m.min(alt)));
                max_altitude = Some(max_altitude.map_or(alt, |m: f64| m.max(alt)));
            }
        });
        if !seen {
            return None;
        }
        Some(BoundingBox {
            west,
            south,
            east,
            north,
            min_altitude,
            max_altitude,
        })
    }

    /// Test whether `point` lies inside this geometry.
    ///
    /// Implemented for `Polygon` and `MultiPolygon` using an even-odd ray-casting test.
    /// Holes are honoured: a point inside the exterior ring but inside any interior ring
    /// is not contained. Points exactly on an edge or vertex may be reported either way,
    /// as is usual for floating-point ray casting. Every other geometry type returns
    /// `false`.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_geo_geojson::{parse, GeoJson, Position};
    ///
    /// let geo = parse(r#"{"type":"Polygon","coordinates":[[[0,0],[4,0],[4,4],[0,4],[0,0]]]}"#).unwrap();
    /// let GeoJson::Geometry(geom) = geo else { panic!("expected geometry") };
    /// assert!(geom.contains(&Position::new(vec![2.0, 2.0]).unwrap()));
    /// assert!(!geom.contains(&Position::new(vec![9.0, 9.0]).unwrap()));
    /// ```
    pub fn contains(&self, point: &Position) -> bool {
        match self {
            Self::Polygon { coordinates } => polygon_contains(coordinates, point),
            Self::MultiPolygon { coordinates } => coordinates
                .iter()
                .any(|polygon| polygon_contains(polygon, point)),
            _ => false,
        }
    }

    fn for_each_position(&self, f: &mut impl FnMut(&Position)) {
        match self {
            Self::Point { coordinates } => f(coordinates),
            Self::MultiPoint { coordinates } | Self::LineString { coordinates } => {
                for p in coordinates {
                    f(p);
                }
            }
            Self::MultiLineString { coordinates } | Self::Polygon { coordinates } => {
                for line in coordinates {
                    for p in line {
                        f(p);
                    }
                }
            }
            Self::MultiPolygon { coordinates } => {
                for polygon in coordinates {
                    for ring in polygon {
                        for p in ring {
                            f(p);
                        }
                    }
                }
            }
            Self::GeometryCollection { geometries } => {
                for geometry in geometries {
                    geometry.for_each_position(f);
                }
            }
        }
    }
}

fn polygon_contains(rings: &[Vec<Position>], point: &Position) -> bool {
    let Some(exterior) = rings.first() else {
        return false;
    };
    if !ring_contains(exterior, point) {
        return false;
    }
    !rings[1..].iter().any(|hole| ring_contains(hole, point))
}

fn ring_contains(ring: &[Position], point: &Position) -> bool {
    if ring.len() < 3 {
        return false;
    }
    let (x, y) = (point.longitude(), point.latitude());
    let mut inside = false;
    let mut j = ring.len() - 1;
    for i in 0..ring.len() {
        let (xi, yi) = (ring[i].longitude(), ring[i].latitude());
        let (xj, yj) = (ring[j].longitude(), ring[j].latitude());
        if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
            inside = !inside;
        }
        j = i;
    }
    inside
}

// ---- Feature types ----

/// A GeoJSON Feature.
#[derive(Debug, Clone, PartialEq)]
pub struct Feature {
    /// The feature's geometry, if any.
    pub geometry: Option<Geometry>,
    /// Arbitrary properties associated with the feature.
    pub properties: Option<Value>,
    /// An optional feature identifier.
    pub id: Option<Value>,
    /// An optional bounding box `[west, south, east, north]` (plus optional altitude pairs).
    pub bbox: Option<Vec<f64>>,
    /// Non-standard members present on the feature, preserved across a serialize/parse round-trip.
    ///
    /// Ordered by key so serialization is deterministic.
    pub foreign_members: BTreeMap<String, Value>,
}

impl Feature {
    /// Re-run the strict validation pass on a (possibly hand-constructed) feature.
    ///
    /// Validates the `bbox`, if any, and the geometry with [`Geometry::validate`].
    pub fn validate(&self) -> Result<(), GeoError> {
        validate_feature_at(self, "")
    }
}

impl Serialize for Feature {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_map(None)?;
        state.serialize_entry("type", "Feature")?;
        state.serialize_entry("geometry", &self.geometry)?;
        state.serialize_entry("properties", &self.properties)?;
        if let Some(id) = &self.id {
            state.serialize_entry("id", id)?;
        }
        if let Some(bbox) = &self.bbox {
            state.serialize_entry("bbox", bbox)?;
        }
        for (k, v) in &self.foreign_members {
            state.serialize_entry(k, v)?;
        }
        state.end()
    }
}

/// A GeoJSON FeatureCollection.
#[derive(Debug, Clone, PartialEq)]
pub struct FeatureCollection {
    /// The features in this collection.
    pub features: Vec<Feature>,
    /// An optional bounding box `[west, south, east, north]` (plus optional altitude pairs).
    pub bbox: Option<Vec<f64>>,
    /// Non-standard members present on the collection, preserved across a serialize/parse round-trip.
    ///
    /// Ordered by key so serialization is deterministic.
    pub foreign_members: BTreeMap<String, Value>,
}

impl FeatureCollection {
    /// Re-run the strict validation pass on a (possibly hand-constructed) collection.
    ///
    /// Validates the collection `bbox`, if any, and every contained feature.
    pub fn validate(&self) -> Result<(), GeoError> {
        validate_feature_collection_at(self, "")
    }
}

impl Serialize for FeatureCollection {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_map(None)?;
        state.serialize_entry("type", "FeatureCollection")?;
        state.serialize_entry("features", &self.features)?;
        if let Some(bbox) = &self.bbox {
            state.serialize_entry("bbox", bbox)?;
        }
        for (k, v) in &self.foreign_members {
            state.serialize_entry(k, v)?;
        }
        state.end()
    }
}

/// A bare top-level Geometry that also carries a `bbox` and/or foreign members.
///
/// [`parse`] produces [`GeoJson::Geometry`] for a plain geometry object and this type
/// only when the object has a `bbox` or non-standard members, so both survive a
/// parse → serialize → parse round-trip.
#[derive(Debug, Clone, PartialEq)]
pub struct GeometryObject {
    /// The geometry itself.
    pub geometry: Geometry,
    /// An optional bounding box `[west, south, east, north]` (plus optional altitude pairs).
    pub bbox: Option<Vec<f64>>,
    /// Non-standard members present on the geometry object, preserved across a
    /// serialize/parse round-trip.
    ///
    /// Ordered by key so serialization is deterministic.
    pub foreign_members: BTreeMap<String, Value>,
}

impl GeometryObject {
    /// Re-run the strict validation pass on a (possibly hand-constructed) geometry object.
    ///
    /// Validates the `bbox`, if any, and the geometry with [`Geometry::validate`].
    pub fn validate(&self) -> Result<(), GeoError> {
        validate_geometry_object_at(self, "")
    }
}

impl Serialize for GeometryObject {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let value = serde_json::to_value(&self.geometry).map_err(serde::ser::Error::custom)?;
        let Value::Object(members) = value else {
            return Err(serde::ser::Error::custom(
                "geometry must serialize to an object",
            ));
        };
        let mut state = serializer.serialize_map(None)?;
        for (k, v) in &members {
            state.serialize_entry(k, v)?;
        }
        if let Some(bbox) = &self.bbox {
            state.serialize_entry("bbox", bbox)?;
        }
        for (k, v) in &self.foreign_members {
            state.serialize_entry(k, v)?;
        }
        state.end()
    }
}

/// The top-level GeoJSON object.
#[derive(Debug, Clone, PartialEq)]
pub enum GeoJson {
    /// A single Feature.
    Feature(Feature),
    /// A collection of Features.
    FeatureCollection(FeatureCollection),
    /// A bare Geometry.
    Geometry(Geometry),
    /// A bare Geometry carrying a `bbox` and/or foreign members.
    GeometryObject(GeometryObject),
}

impl GeoJson {
    /// Re-run the strict validation pass on a (possibly hand-constructed) value.
    ///
    /// [`parse`] already validates, so this is only needed for values you build or
    /// mutate yourself. It applies exactly the same checks: positions must have 2 or 3
    /// finite coordinates within longitude ∈ [-180, 180] and latitude ∈ [-90, 90],
    /// polygon rings must have at least 4 positions and be exactly closed, and any
    /// `bbox` must be a 4- or 6-element array of finite, in-range numbers.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_geo_geojson::{GeoJson, Geometry, Position};
    ///
    /// let geo = GeoJson::Geometry(Geometry::LineString {
    ///     coordinates: vec![
    ///         Position::new(vec![0.0, 0.0]).unwrap(),
    ///         Position::new(vec![1.0, 1.0]).unwrap(),
    ///     ],
    /// });
    /// assert!(geo.validate().is_ok());
    /// ```
    pub fn validate(&self) -> Result<(), GeoError> {
        match self {
            Self::Feature(f) => validate_feature_at(f, ""),
            Self::FeatureCollection(fc) => validate_feature_collection_at(fc, ""),
            Self::Geometry(g) => validate_geometry_at(g, ""),
            Self::GeometryObject(g) => validate_geometry_object_at(g, ""),
        }
    }
}

impl Serialize for GeoJson {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Feature(f) => f.serialize(serializer),
            Self::FeatureCollection(fc) => fc.serialize(serializer),
            Self::Geometry(g) => g.serialize(serializer),
            Self::GeometryObject(g) => g.serialize(serializer),
        }
    }
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

/// Serialize this GeoJSON value back to a compact JSON string.
///
/// The output is valid GeoJSON: [`Feature`] and [`FeatureCollection`] include
/// their `"type"` member, and any captured `bbox`/foreign members are preserved.
///
/// # Example
///
/// ```
/// use tpt_geo_geojson::parse;
///
/// let geo = parse(r#"{"type":"Point","coordinates":[1.0,2.0]}"#).unwrap();
/// let json = tpt_geo_geojson::to_json(&geo).unwrap();
/// assert!(json.contains("\"type\":\"Point\""));
/// ```
pub fn to_json(value: &GeoJson) -> Result<String, GeoError> {
    serde_json::to_string(value).map_err(|e| GeoError {
        kind: GeoErrorKind::Json(e),
        path: String::new(),
    })
}

// ---- Internal parsing ----

fn child(path: &str, member: &str) -> String {
    if path.is_empty() {
        member.to_owned()
    } else {
        format!("{}.{}", path, member)
    }
}

fn index(path: &str, i: usize) -> String {
    format!("{}[{}]", path, i)
}

/// Extract the optional `bbox` array and any non-standard members from `v`,
/// excluding the `known` field names. Used to preserve `bbox` and foreign
/// members across a parse/serialize round-trip.
type CollectedExtra = (Option<Vec<f64>>, BTreeMap<String, Value>);

fn collect_extra(v: &Value, known: &[&str], path: &str) -> Result<CollectedExtra, GeoError> {
    let bbox_path = child(path, "bbox");
    let bbox = match v.get("bbox") {
        None => None,
        Some(b) => {
            let arr = b.as_array().ok_or_else(|| GeoError {
                kind: GeoErrorKind::MalformedCoordinates("bbox must be an array".into()),
                path: bbox_path.clone(),
            })?;
            let mut vals = Vec::with_capacity(arr.len());
            for (i, item) in arr.iter().enumerate() {
                let n = item.as_f64().ok_or_else(|| GeoError {
                    kind: GeoErrorKind::MalformedCoordinates(format!(
                        "bbox element {} must be a number, got {}",
                        i, item
                    )),
                    path: bbox_path.clone(),
                })?;
                vals.push(n);
            }
            validate_bbox_values(&vals, &bbox_path)?;
            Some(vals)
        }
    };
    let foreign = v
        .as_object()
        .map(|obj| {
            obj.iter()
                .filter(|(k, _)| !known.contains(&k.as_str()))
                .map(|(k, val)| (k.clone(), val.clone()))
                .collect::<BTreeMap<String, Value>>()
        })
        .unwrap_or_default();
    Ok((bbox, foreign))
}

fn parse_value(v: &Value, path: &str) -> Result<GeoJson, GeoError> {
    let type_str = v
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| GeoError {
            kind: GeoErrorKind::InvalidType("missing or non-string 'type' field".into()),
            path: child(path, "type"),
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
                        path: child(path, "features"),
                    })?;
            let features_path = child(path, "features");
            let mut features = Vec::with_capacity(features_val.len());
            for (i, fv) in features_val.iter().enumerate() {
                features.push(parse_feature(fv, &index(&features_path, i))?);
            }
            let (bbox, foreign_members) = collect_extra(v, &["type", "features", "bbox"], path)?;
            Ok(GeoJson::FeatureCollection(FeatureCollection {
                features,
                bbox,
                foreign_members,
            }))
        }
        "Feature" => Ok(GeoJson::Feature(parse_feature(v, path)?)),
        _ => {
            let geometry = parse_geometry(v, path)?;
            let known: &[&str] = match geometry {
                Geometry::GeometryCollection { .. } => &["type", "geometries", "bbox"],
                _ => &["type", "coordinates", "bbox"],
            };
            let (bbox, foreign_members) = collect_extra(v, known, path)?;
            if bbox.is_none() && foreign_members.is_empty() {
                Ok(GeoJson::Geometry(geometry))
            } else {
                Ok(GeoJson::GeometryObject(GeometryObject {
                    geometry,
                    bbox,
                    foreign_members,
                }))
            }
        }
    }
}

fn parse_feature(v: &Value, path: &str) -> Result<Feature, GeoError> {
    if v.get("type").and_then(Value::as_str) != Some("Feature") {
        return Err(GeoError {
            kind: GeoErrorKind::InvalidType(format!("expected 'Feature', got {:?}", v.get("type"))),
            path: child(path, "type"),
        });
    }

    let geometry = match v.get("geometry") {
        None | Some(Value::Null) => None,
        Some(geom_val) => Some(parse_geometry(geom_val, &child(path, "geometry"))?),
    };

    let properties = v.get("properties").cloned();
    let id = v.get("id").cloned();
    let (bbox, foreign_members) =
        collect_extra(v, &["type", "geometry", "properties", "id", "bbox"], path)?;

    Ok(Feature {
        geometry,
        properties,
        id,
        bbox,
        foreign_members,
    })
}

fn parse_geometry(v: &Value, path: &str) -> Result<Geometry, GeoError> {
    let type_str = v
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| GeoError {
            kind: GeoErrorKind::InvalidType("geometry missing 'type' field".into()),
            path: child(path, "type"),
        })?;

    let coords_path = child(path, "coordinates");
    match type_str {
        "Point" => {
            let raw = coords_raw(v, &coords_path)?;
            let pos = parse_position(raw, &coords_path)?;
            Ok(Geometry::Point { coordinates: pos })
        }
        "MultiPoint" => {
            let arr = coords_raw(v, &coords_path)?;
            let positions = parse_position_array(arr, &coords_path)?;
            Ok(Geometry::MultiPoint {
                coordinates: positions,
            })
        }
        "LineString" => {
            let arr = coords_raw(v, &coords_path)?;
            let positions = parse_position_array(arr, &coords_path)?;
            Ok(Geometry::LineString {
                coordinates: positions,
            })
        }
        "MultiLineString" => {
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
                lines.push(parse_position_array(line_val, &index(&coords_path, i))?);
            }
            Ok(Geometry::MultiLineString { coordinates: lines })
        }
        "Polygon" => {
            let rings = parse_rings(v, &coords_path)?;
            Ok(Geometry::Polygon { coordinates: rings })
        }
        "MultiPolygon" => {
            let arr = coords_raw(v, &coords_path)?
                .as_array()
                .ok_or_else(|| GeoError {
                    kind: GeoErrorKind::MalformedCoordinates("expected array of polygons".into()),
                    path: coords_path.clone(),
                })?;
            let mut polys = Vec::with_capacity(arr.len());
            for (i, poly_val) in arr.iter().enumerate() {
                polys.push(parse_rings_value(poly_val, &index(&coords_path, i))?);
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
                    path: child(path, "geometries"),
                })?;
            let geoms_path = child(path, "geometries");
            let mut geoms = Vec::with_capacity(geoms_val.len());
            for (i, gv) in geoms_val.iter().enumerate() {
                geoms.push(parse_geometry(gv, &index(&geoms_path, i))?);
            }
            Ok(Geometry::GeometryCollection { geometries: geoms })
        }
        other => Err(GeoError {
            kind: GeoErrorKind::InvalidType(format!("unknown geometry type '{}'", other)),
            path: child(path, "type"),
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
    let coords: Vec<f64> = arr
        .iter()
        .map(|n| {
            n.as_f64().ok_or_else(|| GeoError {
                kind: GeoErrorKind::MalformedCoordinates("coordinate must be a number".into()),
                path: path.to_owned(),
            })
        })
        .collect::<Result<Vec<f64>, _>>()?;
    validate_coords(&coords, path)?;
    Position::new(coords).map_err(|e| GeoError {
        kind: e.kind,
        path: path.to_owned(),
    })
}

fn parse_position_array(v: &Value, path: &str) -> Result<Vec<Position>, GeoError> {
    let arr = v.as_array().ok_or_else(|| GeoError {
        kind: GeoErrorKind::MalformedCoordinates("expected array of positions".into()),
        path: path.to_owned(),
    })?;
    let mut positions = Vec::with_capacity(arr.len());
    for (i, pv) in arr.iter().enumerate() {
        positions.push(parse_position(pv, &index(path, i))?);
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
        let rp = index(path, i);
        let positions = parse_position_array(ring_val, &rp)?;
        validate_ring(&positions, &rp)?;
        rings.push(positions);
    }
    Ok(rings)
}

// ---- Internal validation ----

fn validate_coords(coords: &[f64], path: &str) -> Result<(), GeoError> {
    if coords.len() < 2 || coords.len() > 3 {
        return Err(GeoError {
            kind: GeoErrorKind::MalformedCoordinates(format!(
                "position must have 2 or 3 elements, got {}",
                coords.len()
            )),
            path: path.to_owned(),
        });
    }
    for (i, c) in coords.iter().enumerate() {
        if !c.is_finite() {
            return Err(GeoError {
                kind: GeoErrorKind::MalformedCoordinates(format!(
                    "coordinate {} must be a finite number, got {}",
                    i, c
                )),
                path: path.to_owned(),
            });
        }
    }
    let (lon, lat) = (coords[0], coords[1]);
    if !(-180.0..=180.0).contains(&lon) || !(-90.0..=90.0).contains(&lat) {
        return Err(GeoError {
            kind: GeoErrorKind::MalformedCoordinates(format!(
                "coordinate out of range: longitude {}, latitude {} (must be lon ∈ [-180,180], lat ∈ [-90,90])",
                lon, lat
            )),
            path: path.to_owned(),
        });
    }
    Ok(())
}

fn validate_position(p: &Position, path: &str) -> Result<(), GeoError> {
    validate_coords(&p.0, path)
}

fn validate_ring(ring: &[Position], path: &str) -> Result<(), GeoError> {
    for (i, p) in ring.iter().enumerate() {
        validate_position(p, &index(path, i))?;
    }
    if ring.len() < 4 {
        return Err(GeoError {
            kind: GeoErrorKind::InvalidRing(format!(
                "ring must have at least 4 positions, got {}",
                ring.len()
            )),
            path: path.to_owned(),
        });
    }
    if ring[0] != ring[ring.len() - 1] {
        return Err(GeoError {
            kind: GeoErrorKind::InvalidRing(
                "polygon ring is not closed (first position != last position)".into(),
            ),
            path: path.to_owned(),
        });
    }
    Ok(())
}

fn validate_bbox_values(vals: &[f64], path: &str) -> Result<(), GeoError> {
    if vals.len() != 4 && vals.len() != 6 {
        return Err(GeoError {
            kind: GeoErrorKind::MalformedCoordinates(format!(
                "bbox must have 4 (2D) or 6 (3D) elements, got {}",
                vals.len()
            )),
            path: path.to_owned(),
        });
    }
    for (i, n) in vals.iter().enumerate() {
        if !n.is_finite() {
            return Err(GeoError {
                kind: GeoErrorKind::MalformedCoordinates(format!(
                    "bbox element {} must be a finite number, got {}",
                    i, n
                )),
                path: path.to_owned(),
            });
        }
    }
    let half = vals.len() / 2;
    for corner in [0usize, half] {
        let (lon, lat) = (vals[corner], vals[corner + 1]);
        if !(-180.0..=180.0).contains(&lon) || !(-90.0..=90.0).contains(&lat) {
            return Err(GeoError {
                kind: GeoErrorKind::MalformedCoordinates(format!(
                    "bbox out of range: longitude {}, latitude {} (must be lon ∈ [-180,180], lat ∈ [-90,90])",
                    lon, lat
                )),
                path: path.to_owned(),
            });
        }
    }
    Ok(())
}

fn validate_bbox(bbox: &Option<Vec<f64>>, path: &str) -> Result<(), GeoError> {
    match bbox {
        None => Ok(()),
        Some(vals) => validate_bbox_values(vals, &child(path, "bbox")),
    }
}

fn validate_geometry_at(g: &Geometry, path: &str) -> Result<(), GeoError> {
    let coords_path = child(path, "coordinates");
    match g {
        Geometry::Point { coordinates } => validate_position(coordinates, &coords_path),
        Geometry::MultiPoint { coordinates } | Geometry::LineString { coordinates } => {
            for (i, p) in coordinates.iter().enumerate() {
                validate_position(p, &index(&coords_path, i))?;
            }
            Ok(())
        }
        Geometry::MultiLineString { coordinates } => {
            for (i, line) in coordinates.iter().enumerate() {
                let lp = index(&coords_path, i);
                for (j, p) in line.iter().enumerate() {
                    validate_position(p, &index(&lp, j))?;
                }
            }
            Ok(())
        }
        Geometry::Polygon { coordinates } => {
            for (i, ring) in coordinates.iter().enumerate() {
                validate_ring(ring, &index(&coords_path, i))?;
            }
            Ok(())
        }
        Geometry::MultiPolygon { coordinates } => {
            for (i, polygon) in coordinates.iter().enumerate() {
                let pp = index(&coords_path, i);
                for (j, ring) in polygon.iter().enumerate() {
                    validate_ring(ring, &index(&pp, j))?;
                }
            }
            Ok(())
        }
        Geometry::GeometryCollection { geometries } => {
            let gp = child(path, "geometries");
            for (i, geometry) in geometries.iter().enumerate() {
                validate_geometry_at(geometry, &index(&gp, i))?;
            }
            Ok(())
        }
    }
}

fn validate_geometry_object_at(g: &GeometryObject, path: &str) -> Result<(), GeoError> {
    validate_bbox(&g.bbox, path)?;
    validate_geometry_at(&g.geometry, path)
}

fn validate_feature_at(f: &Feature, path: &str) -> Result<(), GeoError> {
    validate_bbox(&f.bbox, path)?;
    match &f.geometry {
        None => Ok(()),
        Some(g) => validate_geometry_at(g, &child(path, "geometry")),
    }
}

fn validate_feature_collection_at(fc: &FeatureCollection, path: &str) -> Result<(), GeoError> {
    validate_bbox(&fc.bbox, path)?;
    let features_path = child(path, "features");
    for (i, f) in fc.features.iter().enumerate() {
        validate_feature_at(f, &index(&features_path, i))?;
    }
    Ok(())
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

    #[test]
    fn position_constructor_rejects_short_vector() {
        assert!(Position::new(vec![1.0]).is_err());
        let p = Position::new(vec![1.0, 2.0]).unwrap();
        assert_eq!(p.longitude(), 1.0);
        assert_eq!(p.latitude(), 2.0);
        assert_eq!(p.altitude(), None);
    }

    #[test]
    fn feature_serializes_type_member() {
        let s = r#"{"type":"Feature","geometry":{"type":"Point","coordinates":[0,0]},"properties":null}"#;
        let geo = parse(s).unwrap();
        let json = to_json(&geo).unwrap();
        assert!(json.contains(r#""type":"Feature""#));
    }

    #[test]
    fn feature_collection_round_trips() {
        let s = r#"{
            "type":"FeatureCollection",
            "features":[
                {"type":"Feature","geometry":{"type":"Point","coordinates":[1,2]},"properties":{"n":1}},
                {"type":"Feature","geometry":null,"properties":null}
            ]
        }"#;
        let geo = parse(s).unwrap();
        let json = to_json(&geo).unwrap();
        let again = parse(&json).unwrap();
        assert_eq!(geo, again);
    }

    #[test]
    fn bbox_is_preserved() {
        let s = r#"{"type":"FeatureCollection","bbox":[0,0,10,10],"features":[]}"#;
        let geo = parse(s).unwrap();
        if let GeoJson::FeatureCollection(fc) = &geo {
            assert_eq!(fc.bbox, Some(vec![0.0, 0.0, 10.0, 10.0]));
        } else {
            panic!("expected FeatureCollection");
        }
        let json = to_json(&geo).unwrap();
        assert!(json.contains(r#""bbox":[0.0,0.0,10.0,10.0]"#));
        assert_eq!(parse(&json).unwrap(), geo);
    }

    #[test]
    fn foreign_members_are_preserved() {
        let s = r#"{"type":"Feature","properties":null,"title":"hello","extra":42}"#;
        let geo = parse(s).unwrap();
        if let GeoJson::Feature(f) = &geo {
            assert_eq!(
                f.foreign_members.get("title"),
                Some(&serde_json::json!("hello"))
            );
            assert_eq!(f.foreign_members.get("extra"), Some(&serde_json::json!(42)));
        } else {
            panic!("expected Feature");
        }
        let json = to_json(&geo).unwrap();
        assert!(json.contains(r#""title":"hello""#));
        assert!(json.contains(r#""extra":42"#));
    }

    #[test]
    fn error_on_out_of_range_coordinate() {
        let s = r#"{"type":"Point","coordinates":[200.0,10.0]}"#;
        let err = parse(s).unwrap_err();
        assert!(matches!(err.kind, GeoErrorKind::MalformedCoordinates(_)));
        let s2 = r#"{"type":"Point","coordinates":[10.0,200.0]}"#;
        assert!(matches!(
            parse(s2).unwrap_err().kind,
            GeoErrorKind::MalformedCoordinates(_)
        ));
    }

    #[test]
    fn error_on_odd_length_bbox() {
        let s = r#"{"type":"FeatureCollection","bbox":[0,0,10],"features":[]}"#;
        let err = parse(s).unwrap_err();
        assert!(matches!(err.kind, GeoErrorKind::MalformedCoordinates(_)));
        assert!(err.path.contains("bbox"));
    }

    #[test]
    fn error_on_non_array_bbox() {
        let s = r#"{"type":"FeatureCollection","bbox":"bad","features":[]}"#;
        let err = parse(s).unwrap_err();
        assert!(matches!(err.kind, GeoErrorKind::MalformedCoordinates(_)));
    }
}
