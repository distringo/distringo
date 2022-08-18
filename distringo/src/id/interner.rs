pub use std::{borrow::Cow, collections::HashMap};

use crate::id::{Interned, Raw};

#[derive(Default)]
pub struct GeoIdInterner<'i> {
	names: HashMap<Cow<'i, str>, u32>,
	strings: Vec<Cow<'i, str>>,
}

impl<'i> GeoIdInterner<'i> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn count(&self) -> usize {
		self.strings.len()
	}

	fn contains_symbol(&self, name: u32) -> bool {
		self.strings.get(name as usize).is_some()
	}

	fn contains(&self, interned: Interned) -> bool {
		self.contains_symbol(interned.into())
	}

	pub fn intern(&mut self, raw: Raw) -> Interned {
		self.intern_raw_string(raw.into()).into()
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

	fn get_entry_raw(&self, name: u32) -> Option<&Cow<str>> {
		self.strings.get(name as usize)
	}

	pub fn get_entry(&self, interned: Interned) -> Option<&Cow<str>> {
		self.get_entry_raw(interned.into())
	}

	pub fn get_entry_str(&self, interned: Interned) -> Option<&str> {
		use core::borrow::Borrow;
		self.get_entry(interned).map(Borrow::borrow)
	}

	fn resolve_inner(&self, interned: Interned) -> Option<Cow<str>> {
		self.get_entry(interned).cloned()
	}

	fn get_string_raw(&self, interned: Interned) -> Option<String> {
		self.resolve_inner(interned).map(String::from)
	}

	pub fn resolve(&self, interned: Interned) -> Option<Raw> {
		self.get_string_raw(interned).map(Into::into)
	}
}

#[test]
fn intern_and_get() {
	// Start with a basic string
	let string = String::from("a string");
	// Convert that into a geoid
	let raw = Raw::from(string.clone());

	let mut interner: GeoIdInterner = GeoIdInterner::new();
	let interned = interner.intern(raw);

	let resolved = interner.resolve(interned);
	assert!(resolved.is_some());
	assert_eq!(String::from(resolved.unwrap()), string);
}

#[test]
fn intern_and_get_multiple() {
	let mut interner = GeoIdInterner::new();

	let geoid_0_0 = interner.intern(String::from("a string").into());
	let geoid_0_1 = interner.intern(String::from("a string").into());
	let geoid_1_0 = interner.intern(String::from("another string").into());
	let geoid_0_2 = interner.intern(String::from("a string").into());
	let geoid_1_1 = interner.intern(String::from("another string").into());

	let ((name_0_0, name_0_1, name_0_2), (name_1_0, name_1_1)) = (
		(
			u32::from(geoid_0_0),
			u32::from(geoid_0_1),
			u32::from(geoid_0_2),
		),
		(u32::from(geoid_1_0), u32::from(geoid_1_1)),
	);

	assert!(name_0_0 == name_0_1 && name_0_1 == name_0_2);
	assert!(name_1_0 == name_1_1);
}

#[cfg(test)]
mod contains_symbol {
	use super::GeoIdInterner;
	use super::{Interned, Raw};

	#[test]
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
