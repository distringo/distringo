#![cfg_attr(debug_assertions, allow(dead_code))]

use std::{
	collections::HashMap,
	fs::{File, OpenOptions},
	io::{Read, Write},
};

use itertools::Itertools;

use rayon::prelude::*;

use geojson::{Feature, FeatureCollection, GeoJson};

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

type FeatureGeometry<'x> = (&'x str, geo::Geometry<f32>);

#[tracing::instrument(skip(feature))]
fn feature_to_geometry(feature: &Feature) -> FeatureGeometry {
	use core::convert::TryInto;

	let feature_id: &str = feature_id(feature).expect("could not determine feature identifier");

	tracing::trace!("Found feature id {}", feature_id);

	let geometry: &geojson::Geometry = (feature.geometry)
		.as_ref()
		.expect("geometry-less feature?!");

	let geometry: geo::Geometry<f32> = (geometry.value)
		.clone()
		.try_into()
		.expect("failed to convert geometry");

	(feature_id, geometry)
}

fn pair_are_adjacent(geometry_a: &geo::Geometry<f32>, geometry_b: &geo::Geometry<f32>) -> bool {
	use geo::{bounding_rect::BoundingRect, intersects::Intersects};

	// In nearly all cases we care about, we should have bounding boxes.
	//
	// It's worth checking these, as we can completely ignore geometries whose
	// bounding boxes don't overlap.
	match (geometry_a.bounding_rect(), geometry_b.bounding_rect()) {
		// If we have bounding rects, check the bounding rects first.  This allows us to refute
		// obviously-separated geometries.
		(Some(bounding_rect_a), Some(bounding_rect_b)) => {
			bounding_rect_a.intersects(&bounding_rect_b) && geometry_a.intersects(geometry_b)
		}
		// As a fallback, we can skip the bounding-rect check.  But, it's worth knowing about
		// the performance hit.
		_ => {
			// TODO(rye): Use a memoized geometry identifier database.
			tracing::warn!("Missing bounding_rect information for at least one geometry");
			geometry_a.intersects(geometry_b)
		}
	}
}

#[tracing::instrument(skip(pair))]
fn geometry_pair_adjacencies<'x>(
	pair: (FeatureGeometry<'x>, FeatureGeometry<'x>),
) -> Option<[(&'x str, &'x str); 2]> {
	let name_a: &str = pair.0 .0;
	let name_b: &str = pair.1 .0;

	let geometry_a = pair.0 .1;
	let geometry_b = pair.1 .1;

	// If geometries A and B are adjacent, then
	if pair_are_adjacent(&geometry_a, &geometry_b) {
		tracing::debug!(%name_a, %name_b, "Found adjacent geometries!");
		Some([(name_a, name_b), (name_b, name_a)])
	} else {
		tracing::trace!(%name_a, %name_b, "Geometries are not adjacent.");
		None
	}
}

#[tracing::instrument(skip(features))]
fn unwrap_feature_geometry(
	features: &[Feature],
) -> impl Iterator<Item = FeatureGeometry<'_>> + Clone {
	features.iter().map(feature_to_geometry)
}

/// Produce an iterator over pairs of type `FeatureGeometry<'_>`.
#[tracing::instrument(skip(features))]
fn generate_feature_pairs(
	features: &[Feature],
) -> impl Iterator<Item = (FeatureGeometry<'_>, FeatureGeometry<'_>)> {
	unwrap_feature_geometry(features).tuple_combinations()
}

fn generate_adjacencies<'a>(
	feature_pairs: impl Iterator<Item = (FeatureGeometry<'a>, FeatureGeometry<'a>)> + Send,
) -> impl ParallelIterator<Item = HashMap<&'a str, Vec<&'a str>>> {
	feature_pairs
		.par_bridge()
		.filter_map(geometry_pair_adjacencies)
		.inspect(|pair| tracing::debug!(?pair, "Inserting pair"))
		.flatten()
		.fold(HashMap::new, |mut map, (id, neighbor)| {
			map.entry(id).or_insert_with(Vec::new).push(neighbor);
			map
		})
}

fn compute_fragments<'a>(
	feature_pairs: impl Iterator<Item = (FeatureGeometry<'a>, FeatureGeometry<'a>)> + Send,
) -> HashMap<&'a str, Vec<&'a str>> {
	tracing::debug!("Computing adjacency pairs in parallel");

	// First, compute all the adjacencies in parallel.
	let result: Vec<HashMap<&str, Vec<&str>>> = generate_adjacencies(feature_pairs).collect();

	tracing::debug!(
		"Collapsing {} adjacency pair sets to a final map",
		result.len()
	);

	// Then, collect all the individual pieces into a single, final, HashMap<id, [neighbors]>
	result.into_iter().flat_map(HashMap::into_iter).fold(
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

#[tracing::instrument(skip(geojson))]
fn geojson_to_adjacency_map(geojson: &GeoJson) -> HashMap<&str, Vec<&str>> {
	let data: &FeatureCollection = match geojson {
		GeoJson::FeatureCollection(fc) => fc,
		_ => panic!("unsupported geojson type"),
	};

	let features: &Vec<Feature> = &data.features;

	let feature_pairs = generate_feature_pairs(features);

	compute_fragments(feature_pairs)
}

#[tracing::instrument]
fn write_adjacency_map(file: &mut File, adjacency_map: HashMap<&str, Vec<&str>>) {
	tracing::debug!(?file, "Writing adjacency map");

	for (lhs, neighbors) in adjacency_map {
		for rhs in neighbors {
			writeln!(file, "{},{}", lhs, rhs).expect("failed to write output");
		}
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

	let adjacency_map = geojson_to_adjacency_map(&data);

	let mut output_file: File = OpenOptions::new()
		.create(true)
		.write(true)
		.truncate(true)
		.open(output_file)
		.expect("failed to open output file for writing");

	tracing::debug!(?output_file, "Opened output file");

	write_adjacency_map(&mut output_file, adjacency_map);
}
