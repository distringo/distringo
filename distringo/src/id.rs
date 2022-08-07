use std::{collections::HashMap, rc::Rc};

mod geoid;
pub use geoid::*;

#[derive(Default)]
struct GeoIdInterner {
	names: std::collections::HashMap<Rc<Box<str>>, u32>,
	strings: Vec<Rc<Box<str>>>,
}

impl GeoIdInterner {
	pub fn new() -> Self {
		Self::default()
	}

	fn contains_symbol(&self, name: u32) -> bool {
		self.strings.get(name as usize).is_some()
	}

	fn contains_geoid(&self, geoid: GeoId) -> bool {
		match geoid {
			GeoId::Interned(name) => self.contains_symbol(name),
			GeoId::Raw(_) => false,
		}
	}

	pub fn intern(&mut self, geoid: GeoId) -> GeoId {
		match geoid {
			GeoId::Raw(str) => GeoId::Interned(self.intern_raw(str)),
			GeoId::Interned(name) => {
				debug_assert!(self.contains_symbol(name));
				geoid
			}
		}
	}

	fn intern_raw(&mut self, string: Box<str>) -> u32 {
		if let Some(&symbol) = self.names.get(&string) {
			return symbol;
		}

		let rc = std::rc::Rc::new(string);

		// Begin critical section. Insert string, then check len.
		self.strings.push(rc.clone());

		let symbol = self.strings.len() as usize - 1;
		// End critical section. Len is now a valid name.

		let symbol: u32 = symbol as u32;

		self.names.insert(rc, symbol);
		symbol
	}

	fn get_entry(&self, name: u32) -> Option<&Rc<Box<str>>> {
		self.strings.get(name as usize)
	}

	fn get_bstr(&self, name: u32) -> Option<Box<str>> {
		self.get_entry(name).map(|rc| (*rc.clone()).clone())
	}

	pub fn get(&self, geoid: GeoId) -> Option<GeoId> {
		match geoid {
			GeoId::Raw(ref _str) => Some(geoid),
			GeoId::Interned(name) => self.get_bstr(name).map(GeoId::Raw),
		}
	}
}

#[cfg(test)]
mod interner {
	use super::{GeoId, GeoIdInterner};

	#[test]
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

		let raw_geoid = interner.get(interned_geoid);
		assert!(raw_geoid.is_some());
		assert!(raw_geoid.unwrap().is_raw());
	}

	#[test]
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
	mod internal {
		use super::GeoIdInterner;

		#[test]
		fn intern_twice_same_reuses_same_id() {
			let mut interner = GeoIdInterner::new();
			assert_eq!(interner.intern_raw("a string".into()), 0);
			assert_eq!(interner.intern_raw("a string".into()), 0);
		}

		#[test]
		fn intern_twice_and_another_generates_separate_ids() {
			let mut interner = GeoIdInterner::new();
			assert_eq!(interner.intern_raw("a string".into()), 0);
			assert_eq!(interner.intern_raw("a string".into()), 0);
			assert_eq!(interner.intern_raw("another string".into()), 1);
		}
	}
}

// If I take a GeoId::Raw(<some string>) -> Interner::intern(raw) => GeoId::Interned ->
