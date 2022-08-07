#[derive(Clone, Copy)]
#[cfg(feature = "new_geoid")]
pub struct Interned(u32);

#[cfg(feature = "new_geoid")]
impl From<u32> for Interned {
	fn from(u32: u32) -> Self {
		Self(u32)
	}
}

#[cfg(feature = "new_geoid")]
impl From<Interned> for u32 {
	fn from(interned: Interned) -> Self {
		interned.0
	}
}

#[cfg(feature = "new_geoid")]
#[derive(Clone)]
pub struct Raw(String);

#[cfg(feature = "new_geoid")]
impl From<String> for Raw {
	fn from(s: String) -> Self {
		Self(s)
	}
}

#[cfg(feature = "new_geoid")]
impl From<Raw> for String {
	fn from(raw: Raw) -> Self {
		raw.0
	}
}

#[cfg(feature = "new_geoid")]
impl core::ops::Deref for Raw {
	type Target = String;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[derive(Clone, Debug)]
#[cfg(not(feature = "new_geoid"))]
#[deprecated]
pub enum GeoId {
	Interned(u32),
	Raw(String),
}

#[cfg(not(feature = "new_geoid"))]
impl From<String> for GeoId {
	fn from(s: String) -> Self {
		Self::Raw(s)
	}
}

#[cfg(test)]
#[cfg(not(feature = "new_geoid"))]
mod from {
	use super::GeoId;

	#[test]
	fn string() {
		let string: String = String::from("hello, world!");
		let result: GeoId = GeoId::from(string);
		assert!(result.is_raw());
	}
}

#[cfg(not(feature = "new_geoid"))]
impl GeoId {
	#[deprecated]
	pub const fn is_interned(&self) -> bool {
		match self {
			Self::Interned(_) => true,
			Self::Raw(_) => false,
		}
	}

	#[deprecated]
	pub const fn is_raw(&self) -> bool {
		match self {
			Self::Raw(_) => true,
			Self::Interned(_) => false,
		}
	}
}

#[cfg(not(feature = "new_geoid"))]
#[test]
fn is_interned() {
	let geoid = GeoId::Interned(0_u32);
	assert!(geoid.is_interned());
	let geoid = GeoId::Raw("".into());
	assert!(!geoid.is_interned());
}

#[cfg(not(feature = "new_geoid"))]
#[test]
fn is_raw() {
	let geoid = GeoId::Interned(0_u32);
	assert!(!geoid.is_raw());
	let geoid = GeoId::Raw("".into());
	assert!(geoid.is_raw());
}
