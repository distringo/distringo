#[derive(Clone, Copy)]
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

#[cfg(test)]
mod raw {
	use super::Raw;

	#[cfg(test)]
	mod from {
		use super::Raw;

		#[test]
		fn string() {
			let string: String = String::from("hello, world!");
			let result: Raw = Raw::from(string.clone());
			assert_eq!(result.0, string);
		}
	}
}

#[derive(Clone)]
pub struct Raw(String);

impl From<String> for Raw {
	fn from(s: String) -> Self {
		Self(s)
	}
}

impl From<Raw> for String {
	fn from(raw: Raw) -> Self {
		raw.0
	}
}
