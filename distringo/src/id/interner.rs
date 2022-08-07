#[cfg(not(feature = "new_geoid"))]
pub use super::GeoId;
pub use std::{borrow::Cow, collections::HashMap};

#[cfg(feature = "new_geoid")]
use crate::id::geoid::{Interned, Raw};

#[derive(Default)]
pub struct GeoIdInterner<'i> {
	names: HashMap<Cow<'i, str>, u32>,
	strings: Vec<Cow<'i, str>>,
}

impl<'i> GeoIdInterner<'i> {
	pub fn new() -> Self {
		Self::default()
	}

	fn contains_symbol(&self, name: u32) -> bool {
		self.strings.get(name as usize).is_some()
	}

	#[cfg(feature = "new_geoid")]
	fn contains(&self, interned: Interned) -> bool {
		self.contains_symbol(interned.into())
	}

	#[cfg(not(feature = "new_geoid"))]
	fn contains_geoid(&self, geoid: GeoId) -> bool {
		match geoid {
			GeoId::Interned(name) => self.contains_symbol(name),
			GeoId::Raw(_) => false,
		}
	}

	#[cfg(feature = "new_geoid")]
	pub fn intern(&mut self, raw: super::geoid::Raw) -> super::geoid::Interned {
		self.intern_raw_string(raw.to_string()).into()
	}

	#[cfg(not(feature = "new_geoid"))]
	pub fn intern(&mut self, geoid: GeoId) -> GeoId {
		match geoid {
			GeoId::Raw(str) => GeoId::Interned(self.intern_raw(&str)),
			GeoId::Interned(name) => {
				debug_assert!(self.contains_symbol(name));
				geoid
			}
		}
	}

	fn intern_raw_string(&mut self, string: String) -> u32 {
		if let Some(&symbol) = self.names.get(string.as_str()) {
			return symbol;
		}

		let string: Cow<'i, str> = string.into();

		// Begin critical section. Insert string, then check len.
		self.strings.push(string.clone());

		let symbol = self.strings.len() as usize - 1;
		// End critical section. Len is now a valid name.

		let symbol: u32 = symbol as u32;

		self.names.insert(string, symbol);
		symbol
	}

	fn intern_raw(&mut self, string: &str) -> u32 {
		self.intern_raw_string(String::from(string))
	}

	fn get_entry(&self, name: u32) -> Option<&Cow<str>> {
		self.strings.get(name as usize)
	}

	fn get_cloned(&self, name: u32) -> Option<Cow<str>> {
		self.get_entry(name).cloned()
	}

	fn get_str_raw(&self, name: u32) -> Option<&str> {
		self.get_entry(name).map(|str| str.as_ref())
	}

	#[cfg(not(feature = "new_geoid"))]
	fn get_interned_str(&self, geoid: &GeoId) -> Option<&str> {
		match geoid {
			GeoId::Interned(name) => self.get_str_raw(*name),
			// TODO: This could be smarter? Isn't a GeoId::Raw(String) -> Some(&str) conversion possible?
			GeoId::Raw(_) => None,
		}
	}

	fn get_string_raw(&self, name: u32) -> Option<String> {
		self.get_cloned(name).map(String::from)
	}

	#[cfg(not(feature = "new_geoid"))]
	fn get_string(&self, geoid: GeoId) -> Option<String> {
		match geoid {
			GeoId::Interned(name) => self.get_string_raw(name),
			GeoId::Raw(string) => Some(string),
		}
	}

	#[cfg(feature = "new_geoid")]
	pub fn resolve(&self, interned: Interned) -> Option<Raw> {
		self.get_string_raw(interned.into()).map(Into::into)
	}

	#[cfg(not(feature = "new_geoid"))]
	pub fn resolve(&self, geoid: GeoId) -> Option<GeoId> {
		match geoid {
			GeoId::Raw(ref _str) => Some(geoid),
			GeoId::Interned(name) => self.get_string_raw(name).map(GeoId::Raw),
		}
	}
}

#[test]
#[cfg(not(feature = "new_geoid"))]
fn intern_and_get() {
	// Start with a basic string
	let string = String::from("a string");
	// Convert that into a geoid
	let geoid = GeoId::from(string);
	// It should be raw at this point.
	assert!(geoid.is_raw());

	let mut interner: GeoIdInterner = GeoIdInterner::new();
	let interned_geoid = interner.intern(geoid);
	assert!(interned_geoid.is_interned());

	let raw_geoid = interner.resolve(interned_geoid);
	assert!(raw_geoid.is_some());
	assert!(raw_geoid.unwrap().is_raw());
}

#[test]
#[cfg(feature = "new_geoid")]
fn intern_and_get_multiple() {
	let mut interner = GeoIdInterner::new();

	let geoid_0_0 = interner.intern(String::from("a string").into());
	let geoid_0_1 = interner.intern(String::from("a string").into());
	let geoid_1_0 = interner.intern(String::from("another string").into());
	let geoid_0_2 = interner.intern(String::from("a string").into());
	let geoid_1_1 = interner.intern(String::from("another string").into());

	match (
		(
			u32::from(geoid_0_0),
			u32::from(geoid_0_1),
			u32::from(geoid_0_2),
		),
		(u32::from(geoid_1_0), u32::from(geoid_1_1)),
	) {
		((name_0_0, name_0_1, name_0_2), (name_1_0, name_1_1)) => {
			assert!(name_0_0 == name_0_1 && name_0_1 == name_0_2);
			assert!(name_1_0 == name_1_1);
		}
	}
}

#[test]
#[cfg(not(feature = "new_geoid"))]
fn intern_and_get_multiple() {
	let mut interner = GeoIdInterner::new();

	let geoid_0_0 = interner.intern(GeoId::from(String::from("a string")));
	let geoid_0_1 = interner.intern(GeoId::from(String::from("a string")));
	let geoid_1_0 = interner.intern(GeoId::from(String::from("another string")));
	let geoid_0_2 = interner.intern(GeoId::from(String::from("a string")));
	let geoid_1_1 = interner.intern(GeoId::from(String::from("another string")));

	assert!(geoid_0_0.is_interned());
	assert!(geoid_0_1.is_interned());
	assert!(geoid_0_2.is_interned());
	assert!(geoid_1_0.is_interned());
	assert!(geoid_1_1.is_interned());

	match ((geoid_0_0, geoid_0_1, geoid_0_2), (geoid_1_0, geoid_1_1)) {
		(
			(GeoId::Interned(name_0_0), GeoId::Interned(name_0_1), GeoId::Interned(name_0_2)),
			(GeoId::Interned(name_1_0), GeoId::Interned(name_1_1)),
		) => {
			assert!(name_0_0 == name_0_1 && name_0_1 == name_0_2);
			assert!(name_1_0 == name_1_1);
		}
		_ => unreachable!(),
	}
}

#[cfg(test)]
mod contains_symbol {
	#[cfg(not(feature = "new_geoid"))]
	use super::GeoId;
	use super::GeoIdInterner;
	#[cfg(feature = "new_geoid")]
	use super::{Interned, Raw};

	#[test]
	#[cfg(feature = "new_geoid")]
	fn inserted() {
		let raw = Raw::from(String::from("a string"));

		let mut interner = GeoIdInterner::new();
		let interned: Interned = interner.intern(raw);

		assert!(interner.contains_symbol(interned.into()));
	}

	#[test]
	fn uninserted() {
		let interner = GeoIdInterner::new();

		assert!(!interner.contains_symbol(1657_u32));
		assert!(!interner.contains_symbol(0_u32));
	}
}

#[cfg(test)]
mod intern_raw {
	use super::GeoIdInterner;

	#[test]
	fn intern_twice_same_reuses_same_id() {
		let mut interner = GeoIdInterner::new();
		assert_eq!(interner.intern_raw("a string"), 0);
		assert_eq!(interner.intern_raw("a string"), 0);
	}

	#[test]
	fn intern_twice_and_another_generates_separate_ids() {
		let mut interner = GeoIdInterner::new();
		assert_eq!(interner.intern_raw("a string"), 0);
		assert_eq!(interner.intern_raw("a string"), 0);
		assert_eq!(interner.intern_raw("another string"), 1);
	}
}
