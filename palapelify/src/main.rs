// #![cfg_attr(debug_assertions, allow(dead_code))]

use palapelify::{GeoId, GeometryInterner};

use std::{
	collections::{BTreeMap, BTreeSet},
	fs::{File, OpenOptions},
	io::{Read, Write},
};

use geojson::GeoJson;

#[tracing::instrument(skip(adjacency_map))]
fn write_adjacency_map(file: &mut File, adjacency_map: BTreeMap<&GeoId, BTreeSet<&GeoId>>) {
	tracing::debug!(?file, "Writing adjacency map");

	for (lhs, neighbors) in adjacency_map {
		for rhs in neighbors {
			writeln!(file, "{},{}", lhs, rhs).expect("failed to write output");
		}
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

	interner.load_geojson(data);

	tracing::debug!("Interned data from parsed GeoJson");

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
