#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct GeoId(String);

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

#[cfg(test)]
mod geoid {
	use super::GeoId;
	#[test]
	fn clone() {
		let string: String = "I am a string!".to_string();
		assert!(core::ptr::eq(&string, &string));

		let geoid: GeoId = GeoId(string.clone());
		assert!(!core::ptr::eq(&string, &geoid.0));

		let clone = geoid.clone();
		assert!(!core::ptr::eq(&geoid, &clone));

		assert!(!core::ptr::eq(&geoid.0, &clone.0));

		assert_eq!(&geoid.0, &string);
		assert_eq!(&clone.0, &string);
	}

	#[test]
	fn from_string() {
		let string: String = "I am a string!".to_string();

		let geoid: GeoId = string.clone().into();
		assert_eq!(geoid.0, string);
	}
}

fn feature_id_known(feature: &geojson::Feature) -> Option<&str> {
	const KNOWN_GEOID_KEYS: [&str; 2] = ["GEOID10", "GEOID20"];

	KNOWN_GEOID_KEYS
		.iter()
		.find_map(|&key| feature.property(key).and_then(serde_json::Value::as_str))
}

#[cfg(test)]
mod feature_id_known {
	use super::feature_id_known;
	use geojson::Feature;

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

fn feature_id_unknown(feature: &geojson::Feature) -> Option<&str> {
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
	use super::feature_id_unknown;
	use geojson::Feature;

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

pub fn feature_id(feature: &geojson::Feature) -> Option<&str> {
	feature_id_known(feature).or_else(|| feature_id_unknown(feature))
}

#[cfg(test)]
mod feature_id {
	use super::feature_id;
	use geojson::Feature;

	const REAL_GEOID: &str = "I am definitely a real GEOID";

	fn properties(geoid_keys: &[&str]) -> Option<serde_json::Map<String, serde_json::Value>> {
		let mut map = serde_json::Map::new();

		for key in geoid_keys {
			map.insert(
				key.to_string(),
				serde_json::Value::String(REAL_GEOID.to_string()),
			);
		}

		Some(map)
	}

	fn properties_with_values(
		geoid_keys_and_values: &[(&str, &str)],
	) -> Option<serde_json::Map<String, serde_json::Value>> {
		let mut map = serde_json::Map::new();

		for (key, value) in geoid_keys_and_values {
			map.insert(
				key.to_string(),
				serde_json::Value::String(value.to_string()),
			);
		}

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

	fn feature_with_geoid(geoid_key: &str) -> Feature {
		Feature {
			properties: properties(&[geoid_key]),
			..blank_feature()
		}
	}

	fn feature_with_geoids(geoid_keys: &[&str]) -> Feature {
		Feature {
			properties: properties(geoid_keys),
			..blank_feature()
		}
	}

	fn feature_with_geoids_and_values(geoid_keys_and_values: &[(&str, &str)]) -> Feature {
		Feature {
			properties: properties_with_values(geoid_keys_and_values),
			..blank_feature()
		}
	}

	#[test]
	fn known_geoid10() {
		let feature = feature_with_geoid("GEOID10");
		assert_eq!(feature_id(&feature), Some(REAL_GEOID));
	}

	#[test]
	fn known_geoid20() {
		let feature = feature_with_geoid("GEOID20");
		assert_eq!(feature_id(&feature), Some(REAL_GEOID));
	}

	#[test]
	fn known_geoid10_and_geoid20() {
		let feature = feature_with_geoids_and_values(&[
			("GEOID10", "value for GEOID10"),
			("GEOID20", "value for GEOID20"),
		]);
		assert_eq!(feature_id(&feature), Some("value for GEOID10"));
	}

	#[test]
	fn mixed_known_geoid20_and_unknown_geoid30() {
		let feature = feature_with_geoids_and_values(&[
			("GEOID20", "value for GEOID20"),
			("GEOID30", "value for GEOID30"),
		]);
		assert_eq!(feature_id(&feature), Some("value for GEOID20"));
	}

	#[test]
	fn unknown_geoid30() {
		let feature = feature_with_geoid("GEOID30");
		assert_eq!(feature_id(&feature), Some(REAL_GEOID));
	}

	#[test]
	fn empty_feature() {
		let feature = blank_feature();
		assert_eq!(feature_id(&feature), None);
	}
}

mod point;

use crate::point::GeometryPoint;

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
