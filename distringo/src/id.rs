mod geoid;
use std::{collections::HashMap, rc::Rc};

pub use geoid::*;

#[derive(Default)]
struct GeoIdInterner {
	names: std::collections::HashMap<Rc<Box<str>>, u32>,
	strings: Vec<Rc<Box<str>>>,
}

impl GeoIdInterner {
	fn new() -> Self {
		Self::default()
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

	fn get(&self, name: u32) -> Option<Rc<Box<str>>> {
		self.get_entry(name).map(|rc| rc.clone())
	}

	fn get_cloned(&self, name: u32) -> Option<String> {
		self
			.get_entry(name)
			.map(|rc| (*(*rc)).clone().into_string())
	}
}

#[cfg(test)]
mod interner {

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

	#[test]
	fn intern_and_get() {
		let mut interner = GeoIdInterner::new();

		let result_0 = interner.intern_raw("a string".into());
		let result_1 = interner.intern_raw("another string".into());

		assert_ne!(result_0, result_1);

		let result_0 = interner.get(result_0);
		assert!(result_0.is_some());
		let result_0 = result_0.unwrap();
		let result_0 = &*result_0;
		assert_eq!(*result_0, "a string".into());

		let result_1 = interner.get(result_1);
		assert!(result_1.is_some());
		let result_1 = result_1.unwrap();
		let result_1 = &*result_1;
		assert_eq!(*result_1, "another string".into());

		assert_eq!(interner.get(1675), None);
	}
}

// If I take a GeoId::Raw(<some string>) -> Interner::intern(raw) => GeoId::Interned ->
