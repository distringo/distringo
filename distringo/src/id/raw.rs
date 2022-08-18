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

impl AsRef<str> for Raw {
	fn as_ref(&self) -> &str {
		self.0.as_str()
	}
}
