use crate::feature_id;
use crate::point::GeometryPoint;
use crate::GeoId;

#[derive(Default)]
pub struct GeometryInterner {
	inner: std::collections::HashMap<GeoId, std::collections::HashSet<GeometryPoint>>,
	points_to_geoids: std::collections::HashMap<GeometryPoint, std::collections::HashSet<GeoId>>,
}

impl GeometryInterner {
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	fn get(&self, geoid: &GeoId) -> Option<&std::collections::HashSet<GeometryPoint>> {
		self.inner.get(geoid)
	}

	fn insert(&mut self, geoid: GeoId, geometry: geo::Geometry<f64>) {
		use geo::coords_iter::CoordsIter;

		let points: std::collections::HashSet<GeometryPoint> =
			geometry.coords_iter().map(GeometryPoint::from).collect();

		for point in points.iter() {
			let point: GeometryPoint = point.clone();
			self
				.points_to_geoids
				.entry(point)
				.or_insert_with(std::collections::HashSet::new)
				.insert(geoid.clone());
		}

		self.inner.insert(geoid, points);
	}

	fn geoids(&self) -> impl Iterator<Item = &GeoId> + Clone + Send {
		self.inner.keys()
	}

	fn entries(
		&self,
	) -> impl Iterator<Item = (&GeoId, &std::collections::HashSet<GeometryPoint>)> + Clone + Send {
		self.inner.iter()
	}

	fn points(&self) -> impl Iterator<Item = (&GeometryPoint, &std::collections::HashSet<GeoId>)> {
		self.points_to_geoids.iter()
	}

	pub fn load_geojson(&mut self, geojson: geojson::GeoJson) {
		use core::convert::TryFrom;

		let geometries = match geojson {
			geojson::GeoJson::FeatureCollection(fc) => fc.features.into_iter().filter_map(|feature| {
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
	pub fn compute_adjacencies(
		&self,
	) -> std::collections::BTreeMap<&GeoId, std::collections::BTreeSet<&GeoId>> {
		tracing::info!(
			"Computing adjacencies on {} geoids ({} unique points)",
			self.inner.len(),
			self.points_to_geoids.len()
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
						.collect::<std::collections::HashSet<(&GeoId, &GeoId)>>(),
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
				|mut map, pairs: std::collections::HashSet<(&GeoId, &GeoId)>| {
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
				|mut final_map, (id, neighbors)| {
					final_map
						.entry(id)
						.or_insert_with(std::collections::BTreeSet::new)
						.extend(neighbors);
					final_map
				},
			)
	}
}
