use std::collections::*;
use std::fs::OpenOptions;
use std::io::Read;

use itertools::Itertools;

type FeatureGeometry<'x> = (&'x str, geo::Geometry<f64>);

fn property_key_is_geoid_like(key: &str) -> bool {
	&key[0..6] == "GEOID"
}

fn feature_id(feature: &geojson::Feature) -> Option<&str> {
	const KNOWN_GEOID_KEYS: [&str; 2] = ["GEOID10", "GEOID20"];

	if let Some(str) = KNOWN_GEOID_KEYS
		.iter()
		.find_map(|&key| feature.property(key).and_then(serde_json::Value::as_str))
	{
		Some(str)
	} else {
		// Otherwise, try to be somewhat smart about GEOID types before just iving up.
		if let Some((property, _value)) = feature
			.properties_iter()
			.find(|(key, _)| property_key_is_geoid_like(key))
		{
			// This is slow enough to warrant yelling about it and asking for a GitHub issue.
			tracing::warn!("Found a GEOID-like property called {property}, but this was found by manually searching (which is slow).");

			Some(property)
		} else {
			None
		}
	}
}

fn feature_to_geometry(feature: &geojson::Feature) -> FeatureGeometry {
	use core::convert::TryInto;

	let feature_id: &str = feature_id(feature).expect("could not determine feature identifier");

	let geometry: &geojson::Geometry = (feature.geometry)
		.as_ref()
		.expect("geometry-less feature?!");

	let geometry: geo::Geometry<f64> = (geometry.value)
		.to_owned()
		.try_into()
		.expect("failed to convert geometry");

	(feature_id, geometry)
}

#[tracing::instrument]
fn geometry_pair_to_adjacency_fragments<'x>(
	pair: (FeatureGeometry<'x>, FeatureGeometry<'x>),
) -> Option<Vec<(&'x str, &'x str)>> {
	let name_a: &str = pair.0 .0;
	let name_b: &str = pair.1 .0;

	let overlaps: bool = {
		use geo::bounding_rect::BoundingRect;
		use geo::intersects::Intersects;

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
		Some(vec![(name_a, name_b), (name_b, name_a)])
	} else {
		None
	}
}

fn geojson_to_adjacency_map(geojson: &geojson::GeoJson) -> HashMap<&str, Vec<&str>> {
	let data: &geojson::FeatureCollection = match geojson {
		geojson::GeoJson::FeatureCollection(fc) => fc,
		_ => panic!("unsupported geojson type"),
	};

	let features: &Vec<geojson::Feature> = &data.features;

	let adjacency_map: HashMap<&str, Vec<&str>> = features
		.iter()
		.map(feature_to_geometry)
		.tuple_combinations()
		.filter_map(geometry_pair_to_adjacency_fragments)
		.flatten()
		.fold(HashMap::new(), |mut map, (name, neighbor)| {
			map.entry(name).or_insert_with(Vec::new).push(neighbor);
			map
		});

	adjacency_map
}

fn write_adjacency_map(file: &mut std::fs::File, adjacency_map: HashMap<&str, Vec<&str>>) {
	use std::io::Write;

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

	let mut input_file: std::fs::File = OpenOptions::new()
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

	let data: geojson::GeoJson = input_data
		.parse::<geojson::GeoJson>()
		.expect("failed to parse input as geojson");

	let adjacency_map = geojson_to_adjacency_map(&data);

	let mut output_file = OpenOptions::new()
		.create(true)
		.write(true)
		.truncate(true)
		.open(output_file)
		.expect("failed to open {output_file} for writing");

	write_adjacency_map(&mut output_file, adjacency_map);
}
