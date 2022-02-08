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

fn feature_to_geometry(feature: &Feature) -> FeatureGeometry {
	use core::convert::TryInto;

	let feature_id: &str = feature_id(feature).expect("could not determine feature identifier");

	let geometry: &geojson::Geometry = (feature.geometry)
		.as_ref()
		.expect("geometry-less feature?!");

	let geometry: geo::Geometry<f32> = (geometry.value)
		.clone()
		.try_into()
		.expect("failed to convert geometry");

	(feature_id, geometry)
}

#[tracing::instrument]
fn geometry_pair_to_adjacency_fragments<'x>(
	pair: (FeatureGeometry<'x>, FeatureGeometry<'x>),
) -> Option<[(&'x str, &'x str); 2]> {
	let name_a: &str = pair.0 .0;
	let name_b: &str = pair.1 .0;

	let overlaps: bool = {
		use geo::{bounding_rect::BoundingRect, intersects::Intersects};

		let ls_a = &pair.0 .1;
		let ls_b = &pair.1 .1;

		// In nearly all cases, we should have bounding boxes, so check that they
		// overlap before doing the (more intense) operation of checking each segment
		// in a LineString for intersection.
		match (ls_a.bounding_rect(), ls_b.bounding_rect()) {
			(Some(a_bb), Some(b_bb)) => a_bb.intersects(&b_bb) && ls_a.intersects(ls_b),
			// Fall back on simple LineString intersection checking if we couldn't figure
			// out bounding boxes (e.g. because of an empty LineString? this should be rare.)
			_ => ls_a.intersects(ls_b),
		}
	};

	if overlaps {
		Some([(name_a, name_b), (name_b, name_a)])
	} else {
		None
	}
}

fn generate_feature_pairs(
	features: &[Feature],
) -> impl Iterator<Item = ((&str, geo::Geometry<f32>), (&str, geo::Geometry<f32>))> {
	features
		.iter()
		.map(feature_to_geometry)
		.tuple_combinations()
}

fn generate_adjacencies<'a>(
	feature_pairs: impl Iterator<Item = ((&'a str, geo::Geometry<f32>), (&'a str, geo::Geometry<f32>))>
		+ Send,
) -> impl ParallelIterator<Item = HashMap<&'a str, Vec<&'a str>>> {
	feature_pairs
		.par_bridge()
		.filter_map(geometry_pair_to_adjacency_fragments)
		.flatten()
		.fold(HashMap::new, |mut map, (id, neighbor)| {
			map.entry(id).or_insert_with(Vec::new).push(neighbor);
			map
		})
}

fn compute_fragments<'a>(
	feature_pairs: impl Iterator<Item = ((&'a str, geo::Geometry<f32>), (&'a str, geo::Geometry<f32>))>
		+ Send,
) -> HashMap<&'a str, Vec<&'a str>> {
	// First, compute all the adjacencies in parallel.
	let result: Vec<HashMap<&str, Vec<&str>>> = generate_adjacencies(feature_pairs).collect();

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

fn geojson_to_adjacency_map(geojson: &GeoJson) -> HashMap<&str, Vec<&str>> {
	let data: &FeatureCollection = match geojson {
		GeoJson::FeatureCollection(fc) => fc,
		_ => panic!("unsupported geojson type"),
	};

	let features: &Vec<Feature> = &data.features;

	let feature_pairs = generate_feature_pairs(features);

	compute_fragments(feature_pairs)
}

fn write_adjacency_map(file: &mut File, adjacency_map: HashMap<&str, Vec<&str>>) {
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

	let mut input_file: File = OpenOptions::new()
		.read(true)
		.open(input_file)
		.expect("failed to open input file for reading");

	let input_data: String = {
		let mut string: String = String::new();
		input_file
			.read_to_string(&mut string)
			.expect("failed to read file to string");
		string
	};

	let data: GeoJson = input_data
		.parse::<GeoJson>()
		.expect("failed to parse input as geojson");

	let adjacency_map = geojson_to_adjacency_map(&data);

	let mut output_file: File = OpenOptions::new()
		.create(true)
		.write(true)
		.truncate(true)
		.open(output_file)
		.expect("failed to open output file for writing");

	write_adjacency_map(&mut output_file, adjacency_map);
}
