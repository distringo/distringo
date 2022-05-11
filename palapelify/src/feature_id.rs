fn known(feature: &geojson::Feature) -> Option<&str> {
	const KNOWN_GEOID_KEYS: [&str; 2] = ["GEOID10", "GEOID20"];

	KNOWN_GEOID_KEYS
		.iter()
		.find_map(|&key| feature.property(key).and_then(serde_json::Value::as_str))
}

#[cfg(test)]
mod known {
	use super::known;
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

		assert_eq!(known(&feature), Some(REAL_GEOID));
	}

	#[test]
	fn geoid20() {
		let properties = properties("GEOID20");
		let feature = Feature {
			properties,
			..blank_feature()
		};

		assert_eq!(known(&feature), Some(REAL_GEOID));
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
	fn geoid10() {
		assert!(is_geoid_like("GEOID10"));
	}

	#[test]
	fn geoid20() {
		assert!(is_geoid_like("GEOID20"));
	}

	#[test]
	fn special() {
		assert!(is_geoid_like("GEOID98"));
	}

	#[test]
	fn empty() {
		assert!(!is_geoid_like(""));
	}
}

fn unknown(feature: &geojson::Feature) -> Option<&str> {
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
mod unknown {
	use super::unknown;
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

		assert_eq!(unknown(&feature), Some(REAL_GEOID));
	}

	#[test]
	fn geoid20() {
		let properties = properties("GEOID20");
		let feature = Feature {
			properties,
			..blank_feature()
		};

		assert_eq!(unknown(&feature), Some(REAL_GEOID));
	}
}

pub fn feature_id(feature: &geojson::Feature) -> Option<&str> {
	known(feature).or_else(|| unknown(feature))
}

#[cfg(test)]
mod tests {
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
		let (bbox, properties, geometry, id, foreign_members) = (None, None, None, None, None);

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
