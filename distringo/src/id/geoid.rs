#[derive(Clone, Debug)]
pub enum GeoId {
	Interned(u32),
	Raw(Box<str>),
}

impl From<String> for GeoId {
	fn from(s: String) -> Self {
		Self::Raw(s.into_boxed_str())
	}
}

#[cfg(test)]
mod from {
	use super::GeoId;

	#[test]
	fn string() {
		let string: String = String::from("hello, world!");
		let result: GeoId = GeoId::from(string);
		assert!(result.is_raw());
	}
}

impl GeoId {
	pub const fn is_interned(&self) -> bool {
		match self {
			Self::Interned(_) => true,
			Self::Raw(_) => false,
		}
	}

	pub const fn is_raw(&self) -> bool {
		match self {
			Self::Raw(_) => true,
			Self::Interned(_) => false,
		}
	}
}

#[test]
fn is_interned() {
	let geoid = GeoId::Interned(0_u32);
	assert!(geoid.is_interned());
	let geoid = GeoId::Raw("".into());
	assert!(!geoid.is_interned());
}

#[test]
fn is_raw() {
	let geoid = GeoId::Interned(0_u32);
	assert!(!geoid.is_raw());
	let geoid = GeoId::Raw("".into());
	assert!(geoid.is_raw());
}
