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
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InternedGeoId(SymbolU32);

impl GeoIdInterner {
	pub fn intern(&mut self, string: &str) -> InternedGeoId {
		InternedGeoId(self.inner.get_or_intern(string))
	}

	pub fn resolve(&self, interned: &InternedGeoId) -> Option<&str> {
		self.inner.resolve(interned.0)
	}
}
