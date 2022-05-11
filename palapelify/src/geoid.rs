use string_interner::{backend::SimpleBackend, symbol::SymbolU32, StringInterner};

pub struct GeoIdInterner {
	inner: StringInterner<SimpleBackend<SymbolU32>>,
}

impl Default for GeoIdInterner {
	fn default() -> Self {
		Self {
			inner: StringInterner::new(),
		}
	}
}

impl GeoIdInterner {
	pub fn len(&self) -> usize {
		self.inner.len()
	}

	pub fn intern(&mut self, string: &str) -> InternedGeoId {
		InternedGeoId(self.inner.get_or_intern(string))
	}

	pub fn resolve(&self, interned: InternedGeoId) -> Option<&str> {
		self.inner.resolve(interned.0)
	}
}

#[cfg(test)]
mod interner {
	use super::{GeoIdInterner, InternedGeoId};

	#[cfg(test)]
	mod default {
		use super::GeoIdInterner;

		#[test]
		fn works() {
			let _ = GeoIdInterner::default();
		}
	}

	#[cfg(test)]
	mod intern {
		use string_interner::symbol::SymbolU32;

		use super::{GeoIdInterner, InternedGeoId};

		#[test]
		fn intern_once() {
			use string_interner::Symbol;
			let mut interner = GeoIdInterner::default();
			let id: InternedGeoId = interner.intern("181570111003007");
			assert_eq!(
				id,
				InternedGeoId(SymbolU32::try_from_usize(0_usize).expect("0 is a valid index"))
			);
		}

		#[test]
		fn intern_twice_same_geoid() {
			use string_interner::Symbol;
			let mut interner = GeoIdInterner::default();
			let id_first: InternedGeoId = interner.intern("181570111003007");
			let id_second: InternedGeoId = interner.intern("181570111003007");
			assert_eq!(
				id_first,
				InternedGeoId(SymbolU32::try_from_usize(0_usize).expect("0 is a valid index"))
			);
			assert_eq!(
				id_second,
				InternedGeoId(SymbolU32::try_from_usize(0_usize).expect("0 is a valid index"))
			);
			assert_eq!(id_first, id_second);
			assert_eq!(
				std::mem::size_of::<InternedGeoId>(),
				std::mem::size_of::<u32>()
			);
		}

		#[test]
		fn intern_twice_same_geoid_and_once_more() {
			use string_interner::Symbol;
			let mut interner = GeoIdInterner::default();
			let id_first: InternedGeoId = interner.intern("181570111003007");
			let id_second: InternedGeoId = interner.intern("181570111003007");
			let id_third: InternedGeoId = interner.intern("181570111003006");
			assert_eq!(
				id_first,
				InternedGeoId(SymbolU32::try_from_usize(0_usize).expect("0 is a valid index"))
			);
			assert_eq!(
				id_second,
				InternedGeoId(SymbolU32::try_from_usize(0_usize).expect("0 is a valid index"))
			);
			assert_eq!(
				id_third,
				InternedGeoId(SymbolU32::try_from_usize(1_usize).expect("1 is a valid index"))
			);
			assert_eq!(id_first, id_second);
			assert_eq!(
				std::mem::size_of::<InternedGeoId>(),
				std::mem::size_of::<u32>()
			);
		}
	}

	#[cfg(test)]
	mod len {
		use super::GeoIdInterner;

		#[test]
		fn default_zero() {
			let interner = GeoIdInterner::default();
			assert_eq!(interner.len(), 0_usize);
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(test, derive(Debug))]
#[repr(transparent)]
pub struct InternedGeoId(SymbolU32);
