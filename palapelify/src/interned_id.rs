pub struct GeoidInterner {
	inner: string_interner::StringInterner<string_interner::backend::SimpleBackend<usize>>,
}

#[derive(Clone, Copy)]
pub struct InternedGeoid(usize);

impl GeoidInterner {
	fn intern(&mut self, string: &str) -> InternedGeoid {
		InternedGeoid(self.inner.get_or_intern(string))
	}

	fn resolve(&self, interned: &InternedGeoid) -> Option<&str> {
		self.inner.resolve(interned.0)
	}
}
