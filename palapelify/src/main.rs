// #![cfg_attr(debug_assertions, allow(dead_code))]

use palapelify::{feature_id, GeoId, GeoScalar, GeometryPoint};

use std::{
	collections::{BTreeMap, BTreeSet, HashMap, HashSet},
	fs::{File, OpenOptions},
	io::{Read, Write},
};

use geo::coords_iter::CoordsIter;
use itertools::Itertools;

use rayon::prelude::*;

use geojson::{Feature, GeoJson};

#[derive(Default)]
struct GeometryInterner {
	inner: HashMap<GeoId, HashSet<GeometryPoint>>,
	points_to_geoids: HashMap<GeometryPoint, HashSet<GeoId>>,
}

impl GeometryInterner {
	#[must_use]
	fn new() -> Self {
		Self::default()
	}

	fn get(&self, geoid: &GeoId) -> Option<&HashSet<GeometryPoint>> {
		self.inner.get(geoid)
	}

	fn insert(&mut self, geoid: GeoId, geometry: geo::Geometry<f64>) {
		let points: HashSet<GeometryPoint> = geometry.coords_iter().map(GeometryPoint::from).collect();

		for point in points.iter() {
			let point: GeometryPoint = point.clone();
			self
				.points_to_geoids
				.entry(point)
				.or_insert_with(HashSet::new)
				.insert(geoid.clone());
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

	fn load_geojson(&mut self, geojson: GeoJson) {
		use core::convert::TryFrom;

		let geometries = match geojson {
			GeoJson::FeatureCollection(fc) => fc.features.into_iter().filter_map(|feature| {
				let geoid = feature_id(&feature)
					.map(ToString::to_string)
					.map(GeoId::from);

				let geometry: Option<geo_types::Geometry<f64>> =
					feature.geometry.map(TryFrom::try_from).and_then(Result::ok);

				match (geoid, geometry) {
					(Some(geoid), Some(geometry)) => Some((geoid, geometry)),
					_ => todo!(),
				}
			}),
			_ => todo!(),
		};

		for (geoid, geometry) in geometries {
			self.insert(geoid, geometry)
		}
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
			.filter_map(|(point, containing_geoids)| match containing_geoids.len() {
				2.. => Some(
					containing_geoids
						.iter()
						.permutations(2)
						.map(|permutation| (permutation[0], permutation[1]))
						.collect::<HashSet<(&GeoId, &GeoId)>>(),
				),
				1 => None,
				0 => {
					tracing::warn!("Point has no containing GeoIds");
					None
				}
				_ => unreachable!(),
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
