use std::convert::Infallible;

#[derive(Clone, Debug)]
enum GeoId {
	Interned(u32),
	Raw(Box<str>),
}

impl From<String> for GeoId {
	fn from(s: String) -> Self {
		Self::Raw(s.into_boxed_str())
	}
}

impl GeoId {
	const fn is_interned(&self) -> bool {
		match self {
			Self::Interned(_) => true,
			Self::Raw(_) => false,
		}
	}

	const fn is_raw(&self) -> bool {
		match self {
			Self::Raw(_) => true,
			Self::Interned(_) => false,
		}
	}
}

struct GeoIdInterner {}

impl GeoIdInterner {}

#[cfg(test)]
mod test {
	use super::GeoId;

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
}
