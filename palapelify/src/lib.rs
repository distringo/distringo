#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GeoScalar(i32);

impl From<f64> for GeoScalar {
	fn from(f64: f64) -> Self {
		debug_assert!(f64 < 180.00 && f64 > -180.0);

		Self((f64 * 1E6).trunc() as i32)
	}
}

impl From<GeoScalar> for f64 {
	fn from(geo_scalar: GeoScalar) -> f64 {
		(geo_scalar.0 as f64) / 1E6
	}
}

#[cfg(test)]
mod geoscalar {
	use super::GeoScalar;

	#[cfg(test)]
	mod from_f64 {
		use super::GeoScalar;

		impl core::fmt::Debug for GeoScalar {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				f.debug_tuple("GeoScalar").field(&self.0).finish()
			}
		}

		#[test]
		fn converts_positive_in_range() {
			let degrees: f64 = 87.31275;
			assert_eq!(GeoScalar::from(degrees), GeoScalar(87312750));
		}

		#[test]
		fn converts_negative_in_range() {
			let degrees: f64 = -37.172718;
			assert_eq!(GeoScalar::from(degrees), GeoScalar(-37172718));
		}

		#[test]
		#[should_panic]
		fn asserts_in_range_positive() {
			let degrees: f64 = 180.0;
			let _ = GeoScalar::from(degrees);
		}

		#[test]
		#[should_panic]
		fn asserts_in_range_negative() {
			let degrees: f64 = -180.0;
			let _ = GeoScalar::from(degrees);
		}
	}
}

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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GeometryPoint([GeoScalar; 2]);

impl From<geo::Coordinate<f64>> for GeometryPoint {
	fn from(coordinate: geo::Coordinate<f64>) -> Self {
		GeometryPoint([coordinate.y.into(), coordinate.x.into()])
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
