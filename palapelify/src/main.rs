// #![cfg_attr(debug_assertions, allow(dead_code))]

use std::{
	collections::{BTreeMap, BTreeSet, HashMap, HashSet},
	fs::{File, OpenOptions},
	io::{Read, Write},
};

use geo::coords_iter::CoordsIter;
use itertools::Itertools;

use rayon::prelude::*;

use geojson::{Feature, GeoJson};

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct GeometryPoint([GeoScalar; 2]);

#[derive(Default)]
struct GeometryInterner {
	inner: HashMap<GeoId, HashSet<GeometryPoint>>,
	points_to_geoids: HashMap<GeometryPoint, HashSet<GeoId>>,
}

impl From<geo::Coordinate<f32>> for GeometryPoint {
	fn from(coordinate: geo::Coordinate<f32>) -> Self {
		GeometryPoint([coordinate.y.into(), coordinate.x.into()])
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
		let points: HashSet<GeometryPoint> = geometry.coords_iter().map(GeometryPoint::from).collect();

		for point in points.iter() {
			let point: GeometryPoint = point.clone();
			self
				.points_to_geoids
				.entry(point)
				.or_insert_with(HashSet::new)
				.insert(GeoId(geoid.0.clone()));
		}

		self.inner.insert(geoid, points);
	}

	fn geoids(&self) -> impl Iterator<Item = &GeoId> + Clone + Send {
		self.inner.keys()
	}

	fn entries(&self) -> impl Iterator<Item = (&GeoId, &HashSet<GeometryPoint>)> + Clone + Send {
		self.inner.iter()
	}

	fn points(&self) -> impl Iterator<Item = (&GeometryPoint, &HashSet<GeoId>)> {
		self.points_to_geoids.iter()
	}

	#[tracing::instrument(skip(self))]
	fn compute_adjacencies(&self) -> BTreeMap<&GeoId, BTreeSet<&GeoId>> {
		tracing::info!(
			"Computing adjacencies on {} geoids ({} unique points)",
			self.inner.len(),
			self.points_to_geoids.len()
		);

		let maps = self
			.points()
			.par_bridge()
			.filter_map(|(point, containing_geoids)| {
				if containing_geoids.len() > 1 {
					Some(
						containing_geoids
							.iter()
							.permutations(2)
							.map(|permutation| (permutation[0], permutation[1]))
							.collect::<HashSet<(&GeoId, &GeoId)>>(),
					)
				} else {
					None
				}
			})
			.fold(
				BTreeMap::new,
				|mut map, pairs: HashSet<(&GeoId, &GeoId)>| {
					tracing::debug!("Folding in {} entries", pairs.len());

					for pair in pairs {
						let geoid_a = pair.0;
						let geoid_b = pair.1;

						map
							.entry(geoid_a)
							.or_insert_with(BTreeSet::new)
							.insert(geoid_b);
					}

					map
				},
			)
			.collect::<Vec<_>>();

		tracing::info!("Collected {} individual maps; merging", maps.len());

		maps
			.into_iter()
			.inspect(|map| tracing::debug!("Merging {} entries", map.len()))
			.flat_map(BTreeMap::into_iter)
			.fold(BTreeMap::new(), |mut final_map, (id, neighbors)| {
				final_map
					.entry(id)
					.or_insert_with(BTreeSet::new)
					.extend(neighbors);
				final_map
			})
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
		assert!(is_geoid_like("GEOID10"));
	}

	#[test]
	fn geoid20_geoid_like() {
		assert!(is_geoid_like("GEOID20"));
	}

	#[test]
	fn special_geoid_like() {
		assert!(is_geoid_like("GEOID98"));
	}

	#[test]
	fn empty_geoid_like() {
		assert!(!is_geoid_like(""));
	}
}

fn feature_id_unknown(feature: &Feature) -> Option<&str> {
	if let Some((key, value)) = feature
		.properties_iter()
		.find(|(key, _)| is_geoid_like(key))
	{
		tracing::warn!(%key, "Found a GEOID-like property, but by manual search. Please consider filing an issue to add this to the list of known GEOID keys.");
		value.as_str()
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

#[tracing::instrument(skip(adjacency_map))]
fn write_adjacency_map(file: &mut File, adjacency_map: BTreeMap<&GeoId, BTreeSet<&GeoId>>) {
	tracing::debug!(?file, "Writing adjacency map");

	for (lhs, neighbors) in adjacency_map {
		for rhs in neighbors {
			writeln!(file, "{},{}", lhs, rhs).expect("failed to write output");
		}
	}
}

fn load_geojson(geojson: GeoJson, interner: &mut GeometryInterner) {
	use core::convert::TryFrom;

	let geometries = match geojson {
		GeoJson::FeatureCollection(fc) => fc.features.into_iter().filter_map(|feature| {
			let geoid = feature_id(&feature)
				.map(ToString::to_string)
				.map(GeoId::from);

			let geometry: Option<geo_types::Geometry<f32>> =
				feature.geometry.map(TryFrom::try_from).and_then(Result::ok);

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

#[tracing::instrument(skip(input_file))]
fn process_input_file(input_file: &mut File) -> GeometryInterner {
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

	interner
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

	let interner = process_input_file(&mut input_file);

	let adjacency_map = interner.compute_adjacencies();

	let mut output_file: File = OpenOptions::new()
		.create(true)
		.write(true)
		.truncate(true)
		.open(output_file)
		.expect("failed to open output file for writing");

	tracing::info!(?output_file, "Opened output file");

	write_adjacency_map(&mut output_file, adjacency_map);

	tracing::info!("Finished writing output");
}
