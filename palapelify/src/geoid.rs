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
