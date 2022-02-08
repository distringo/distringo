// #![cfg_attr(debug_assertions, allow(dead_code))]

use std::{
	collections::{HashMap, HashSet},
	fs::{File, OpenOptions},
	io::{Read, Write},
};

use geo::coords_iter::CoordsIter;
use itertools::Itertools;

use rayon::prelude::*;

use geojson::{Feature, GeoJson};

#[derive(Debug, Hash, Eq, PartialEq)]
struct GeoId(String);

impl From<String> for GeoId {
	fn from(string: String) -> Self {
		Self(string)
	}
}

impl core::fmt::Display for GeoId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
struct GeoScalar(i32);

impl From<f32> for GeoScalar {
	fn from(f32: f32) -> Self {
		debug_assert!(f32 < 180.00 && f32 > -180.0);

		Self((f32 * 1E6).trunc() as i32)
	}
}

impl From<GeoScalar> for f32 {
	fn from(geo_scalar: GeoScalar) -> f32 {
		(geo_scalar.0 as f32) / 1E6
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
struct GeometryPoint([GeoScalar; 2]);

#[derive(Default)]
struct GeometryInterner {
	inner: HashMap<GeoId, HashSet<GeometryPoint>>,
}

impl From<geo::Coordinate<f32>> for GeometryPoint {
	fn from(coordinate: geo::Coordinate<f32>) -> Self {
		GeometryPoint([coordinate.x.into(), coordinate.y.into()])
	}
}

impl GeometryInterner {
	#[must_use]
	fn new() -> Self {
		Self::default()
	}

	fn get(&self, geoid: &GeoId) -> Option<&HashSet<GeometryPoint>> {
		self.inner.get(geoid)
	}

	fn insert(&mut self, geoid: GeoId, geometry: geo::Geometry<f32>) {
		let points = geometry.coords_iter().map(GeometryPoint::from).collect();
		self.inner.insert(geoid, points);
	}
}

impl GeometryInterner {
	fn geoids(&self) -> impl Iterator<Item = &GeoId> + Clone + Send {
		self.inner.keys()
	}

	fn entries(&self) -> impl Iterator<Item = (&GeoId, &HashSet<GeometryPoint>)> + Clone + Send {
		self.inner.iter()
	}
}

fn feature_id_known(feature: &Feature) -> Option<&str> {
	const KNOWN_GEOID_KEYS: [&str; 2] = ["GEOID10", "GEOID20"];

	KNOWN_GEOID_KEYS
		.iter()
		.find_map(|&key| feature.property(key).and_then(serde_json::Value::as_str))
}

#[cfg(test)]
mod feature_id_known {
	use super::{feature_id_known, Feature};

	const REAL_GEOID: &str = "A valid geoid!";

	fn properties(key: &str) -> Option<serde_json::Map<String, serde_json::Value>> {
		let mut map = serde_json::Map::new();
		map.insert(
			key.to_string(),
			serde_json::Value::String(REAL_GEOID.to_string()),
		);
		Some(map)
	}

	fn blank_feature() -> Feature {
		let bbox = None;
		let properties = None;
		let geometry = None;
		let id = None;
		let foreign_members = None;

		Feature {
			bbox,
			properties,
			geometry,
			id,
			foreign_members,
		}
	}

	#[test]
	fn geoid10() {
		let properties = properties("GEOID10");
		let feature = Feature {
			properties,
			..blank_feature()
		};

		assert_eq!(feature_id_known(&feature), Some(REAL_GEOID));
	}

	#[test]
	fn geoid20() {
		let properties = properties("GEOID20");
		let feature = Feature {
			properties,
			..blank_feature()
		};

		assert_eq!(feature_id_known(&feature), Some(REAL_GEOID));
	}
}

fn is_geoid_like(key: &str) -> bool {
	match key.get(0..5) {
		Some(str) => str == "GEOID",
		None => false,
	}
}

#[cfg(test)]
mod is_geoid_like {
	use super::is_geoid_like;

	#[test]
	fn geoid10_geoid_like() {
		assert_eq!(is_geoid_like("GEOID10"), true);
	}

	#[test]
	fn geoid20_geoid_like() {
		assert_eq!(is_geoid_like("GEOID20"), true);
	}

	#[test]
	fn special_geoid_like() {
		assert_eq!(is_geoid_like("GEOID98"), true);
	}

	#[test]
	fn empty_geoid_like() {
		assert_eq!(is_geoid_like(""), false);
	}
}

fn feature_id_unknown(feature: &Feature) -> Option<&str> {
	if let Some((key, _value)) = feature
		.properties_iter()
		.find(|(key, _)| is_geoid_like(key))
	{
		tracing::warn!(%key, "Found a GEOID-like property, but by manual search. Please consider filing an issue to add this to the list of known GEOID keys.");
		Some(key)
	} else {
		let properties: Vec<(&String, &serde_json::Value)> = feature.properties_iter().collect();
		tracing::warn!(
			?properties,
			"Failed to find a GEOID-like property by manual search."
		);

		None
	}
}

#[cfg(test)]
mod feature_id_unknown {
	use super::{feature_id_unknown, Feature};

	const REAL_GEOID: &str = "A valid geoid!";

	fn properties(key: &str) -> Option<serde_json::Map<String, serde_json::Value>> {
		let mut map = serde_json::Map::new();
		map.insert(
			key.to_string(),
			serde_json::Value::String(REAL_GEOID.to_string()),
		);
		Some(map)
	}

	fn blank_feature() -> Feature {
		let bbox = None;
		let properties = None;
		let geometry = None;
		let id = None;
		let foreign_members = None;

		Feature {
			bbox,
			properties,
			geometry,
			id,
			foreign_members,
		}
	}

	#[test]
	fn geoid10() {
		let properties = properties("GEOID10");
		let feature = Feature {
			properties,
			..blank_feature()
		};

		assert_eq!(feature_id_unknown(&feature), Some(REAL_GEOID));
	}

	#[test]
	fn geoid20() {
		let properties = properties("GEOID20");
		let feature = Feature {
			properties,
			..blank_feature()
		};

		assert_eq!(feature_id_unknown(&feature), Some(REAL_GEOID));
	}
}

fn feature_id(feature: &Feature) -> Option<&str> {
	feature_id_known(feature).or_else(|| feature_id_unknown(feature))
}

#[tracing::instrument]
fn write_adjacency_map(file: &mut File, adjacency_map: HashMap<&GeoId, Vec<&GeoId>>) {
	tracing::debug!(?file, "Writing adjacency map");

	for (lhs, neighbors) in adjacency_map {
		for rhs in neighbors {
			writeln!(file, "{},{}", lhs, rhs).expect("failed to write output");
		}
	}
}

fn compute_adjacencies(interner: &GeometryInterner) -> HashMap<&GeoId, Vec<&GeoId>> {
	let mut counter = 0_usize;

	let maps: Vec<HashMap<&GeoId, Vec<&GeoId>>> = interner
		.entries()
		.tuple_combinations::<(_, _)>()
		.inspect(|e| {
			counter += 1;
			if counter % 1000 == 0 {
				tracing::debug!("Generated {} tuple combinations", counter)
			}
		})
		.par_bridge()
		.filter_map(|((a_geoid, a_points), (b_geoid, b_points))| {
			let mut intersection = a_points.intersection(&b_points);

			if let Some(_) = intersection.next() {
				Some([(a_geoid, b_geoid), (b_geoid, a_geoid)])
			} else {
				None
			}
		})
		.flatten()
		.fold(HashMap::new, |mut map, (geoid_a, geoid_b)| {
			map.entry(geoid_a).or_insert_with(Vec::new).push(geoid_b);
			map
		})
		.collect();

	maps.into_iter().flat_map(HashMap::into_iter).fold(
		HashMap::new(),
		|mut final_map, (id, neighbors)| {
			final_map
				.entry(id)
				.or_insert_with(Vec::new)
				.extend(neighbors);
			final_map
		},
	)
}

fn load_geojson(geojson: GeoJson, interner: &mut GeometryInterner) {
	use core::convert::TryFrom;

	let geometries = match geojson {
		GeoJson::FeatureCollection(fc) => fc.features.into_iter().filter_map(|feature| {
			let geoid = feature_id(&feature)
				.map(ToString::to_string)
				.map(GeoId::from);

			let geometry: Option<geo_types::Geometry<f32>> = feature
				.geometry
				.map(TryFrom::try_from)
				.map(Result::ok)
				.flatten();

			match (geoid, geometry) {
				(Some(geoid), Some(geometry)) => Some((geoid, geometry)),
				_ => todo!(),
			}
		}),
		_ => todo!(),
	};

	for (geoid, geometry) in geometries {
		interner.insert(geoid, geometry)
	}
}

fn main() {
	tracing_subscriber::fmt::init();

	let input_file: String = std::env::args().nth(1).expect("missing input file name");
	let output_file: String = std::env::args().nth(2).expect("missing output file name");

	tracing::info!(?input_file, ?output_file, "Validated arguments");

	let mut input_file: File = OpenOptions::new()
		.read(true)
		.open(input_file)
		.expect("failed to open input file for reading");

	tracing::debug!(?input_file, "Opened input file");

	let input_data: String = {
		let mut string: String = String::new();
		input_file
			.read_to_string(&mut string)
			.expect("failed to read file to string");
		string
	};

	{
		let input_bytes = input_data.len();
		tracing::debug!(?input_file, "Read {} bytes", input_bytes);
	}

	let data: GeoJson = input_data
		.parse::<GeoJson>()
		.expect("failed to parse input as geojson");

	tracing::debug!(?input_file, "Parsed to GeoJson");

	let mut interner: GeometryInterner = GeometryInterner::new();

	load_geojson(data, &mut interner);

	let adjacency_map = compute_adjacencies(&interner);

	let mut output_file: File = OpenOptions::new()
		.create(true)
		.write(true)
		.truncate(true)
		.open(output_file)
		.expect("failed to open output file for writing");

	tracing::debug!(?output_file, "Opened output file");

	write_adjacency_map(&mut output_file, adjacency_map);
}
