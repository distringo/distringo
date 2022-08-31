use crate::feature_id;
use crate::point::GeometryPoint;

use distringo::{id::GeoIdInterner, id::Interned};

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

#[derive(Default)]
pub struct GeometryInterner<'a> {
	geoid_interner: GeoIdInterner<'a>,
	point_containers: HashMap<GeometryPoint, HashSet<Interned>>,
}

impl<'a> GeometryInterner<'a> {
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	fn insert(&mut self, geoid: Interned, geometry: geo::Geometry<f64>) {
		use geo::coords_iter::CoordsIter;

		let points: HashSet<GeometryPoint> = geometry.coords_iter().map(GeometryPoint::from).collect();

		for point in &points {
			let point: GeometryPoint = point.clone();
			self
				.point_containers
				.entry(point)
				.or_insert_with(HashSet::new)
				.insert(geoid);
		}
	}

	#[must_use = "returns an iterator, which does nothing if not consumed"]
	fn points(&self) -> impl Iterator<Item = (&GeometryPoint, &HashSet<Interned>)> {
		self.point_containers.iter()
	}

	fn process_feature(
		&mut self,
		feature: geojson::Feature,
	) -> Option<(Interned, geo_types::Geometry<f64>)> {
		let feature_geoid = feature_id(&feature).map(|string| string.to_string().into());

		let geoid = feature_geoid.map(|string| self.geoid_interner.intern(string));

		let geometry: Option<geo_types::Geometry<f64>> = feature
			.geometry
			.map(core::convert::TryFrom::try_from)
			.and_then(Result::ok);

		match (geoid, geometry) {
			(Some(geoid), Some(geometry)) => Some((geoid, geometry)),
			_ => None,
		}
	}

	pub fn load_geojson(&mut self, geojson: geojson::GeoJson) {
		match geojson {
			geojson::GeoJson::FeatureCollection(fc) => {
				for feature in fc.features {
					if let Some((geoid, geometry)) = self.process_feature(feature) {
						self.insert(geoid, geometry);
					}
				}
			}
			_ => todo!(),
		}
	}

	#[tracing::instrument(skip(self))]
	pub fn compute_adjacencies(&self) -> BTreeMap<&str, BTreeSet<&str>> {
		use itertools::Itertools;
		use rayon::iter::{ParallelBridge, ParallelIterator};

		tracing::info!(
			"Computing adjacencies on {} geoids ({} unique points)",
			self.geoid_interner.count(),
			self.point_containers.len()
		);

		let maps = self
			.points()
			.par_bridge()
			.filter_map(
				|(_point, containing_geoids)| match containing_geoids.len() {
					2.. => Some(
						containing_geoids
							.iter()
							.permutations(2)
							.map(|permutation| (permutation[0], permutation[1]))
							.collect::<HashSet<(&Interned, &Interned)>>(),
					),
					1 => None,
					0 => {
						tracing::warn!("Point has no containing GeoIds");
						None
					}
					_ => unreachable!(),
				},
			)
			.fold(
				BTreeMap::new,
				|mut map, pairs: HashSet<(&Interned, &Interned)>| {
					tracing::trace!("Folding in {} entries", pairs.len());

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
			.fold(BTreeMap::new(), |mut final_map, (&id, neighbors)| {
				let id = self
					.resolve_geoid(id)
					.expect("attempted to resolve id at final step but id was not interned");

				let neighbors = neighbors
					.iter()
					.filter_map(|&&neighbor_id| self.resolve_geoid(neighbor_id));

				final_map
					.entry(id)
					.or_insert_with(BTreeSet::new)
					.extend(neighbors);
				final_map
			})
	}

	fn resolve_geoid(&self, geoid: Interned) -> Option<&str> {
		self.geoid_interner.get_entry_str(geoid)
	}
}
