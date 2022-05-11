use crate::feature_id;
use crate::geoid::{GeoIdInterner, InternedGeoId};
use crate::point::GeometryPoint;

#[derive(Default)]
pub struct GeometryInterner {
	geoid_interner: GeoIdInterner,
	point_containers:
		std::collections::HashMap<GeometryPoint, std::collections::HashSet<InternedGeoId>>,
}

impl GeometryInterner {
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	fn insert(&mut self, geoid: InternedGeoId, geometry: geo::Geometry<f64>) {
		use geo::coords_iter::CoordsIter;

		let points: std::collections::HashSet<GeometryPoint> =
			geometry.coords_iter().map(GeometryPoint::from).collect();

		for point in points.iter() {
			let point: GeometryPoint = point.clone();
			self
				.point_containers
				.entry(point)
				.or_insert_with(std::collections::HashSet::new)
				.insert(geoid);
		}
	}

	fn points(
		&self,
	) -> impl Iterator<Item = (&GeometryPoint, &std::collections::HashSet<InternedGeoId>)> {
		self.point_containers.iter()
	}

	fn process_feature(
		&mut self,
		feature: geojson::Feature,
	) -> Option<(InternedGeoId, geo_types::Geometry<f64>)> {
		let geoid = feature_id(&feature).map(|string| self.geoid_interner.intern(string));

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
				for feature in fc.features.into_iter() {
					if let Some((geoid, geometry)) = self.process_feature(feature) {
						self.insert(geoid, geometry);
					}
				}
			}
			_ => todo!(),
		}
	}

	#[tracing::instrument(skip(self))]
	pub fn compute_adjacencies(
		&self,
	) -> std::collections::BTreeMap<&str, std::collections::BTreeSet<&str>> {
		tracing::info!(
			"Computing adjacencies on {} geoids ({} unique points)",
			self.geoid_interner.len(),
			self.point_containers.len()
		);

		use itertools::Itertools;
		use rayon::iter::{ParallelBridge, ParallelIterator};

		let maps = self
			.points()
			.par_bridge()
			.filter_map(|(point, containing_geoids)| match containing_geoids.len() {
				2.. => Some(
					containing_geoids
						.iter()
						.permutations(2)
						.map(|permutation| (permutation[0], permutation[1]))
						.collect::<std::collections::HashSet<(&InternedGeoId, &InternedGeoId)>>(),
				),
				1 => None,
				0 => {
					tracing::warn!("Point has no containing GeoIds");
					None
				}
				_ => unreachable!(),
			})
			.fold(
				std::collections::BTreeMap::new,
				|mut map, pairs: std::collections::HashSet<(&InternedGeoId, &InternedGeoId)>| {
					tracing::trace!("Folding in {} entries", pairs.len());

					for pair in pairs {
						let geoid_a = pair.0;
						let geoid_b = pair.1;

						map
							.entry(geoid_a)
							.or_insert_with(std::collections::BTreeSet::new)
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
			.flat_map(std::collections::BTreeMap::into_iter)
			.fold(
				std::collections::BTreeMap::new(),
				|mut final_map, (&id, neighbors)| {
					let id = self
						.resolve_geoid(id)
						.expect("attempted to resolve id at final step but id was not interned");

					let neighbors = neighbors
						.iter()
						.filter_map(|&&neighbor_id| self.resolve_geoid(neighbor_id));

					final_map
						.entry(id)
						.or_insert_with(std::collections::BTreeSet::new)
						.extend(neighbors);
					final_map
				},
			)
	}

	fn resolve_geoid(&self, geoid: InternedGeoId) -> Option<&str> {
		self.geoid_interner.resolve(geoid)
	}
}
