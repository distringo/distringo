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
