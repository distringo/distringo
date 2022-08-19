#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Interned(u32);

impl From<u32> for Interned {
	fn from(u32: u32) -> Self {
		Self(u32)
	}
}

impl From<Interned> for u32 {
	fn from(interned: Interned) -> Self {
		interned.0
	}
}
