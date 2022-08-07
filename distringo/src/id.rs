mod geoid;
pub use geoid::*;

struct GeoIdInterner {
	names: std::collections::HashMap<Box<str>, u32>,
	strings: Vec<Box<str>>,
}

impl GeoIdInterner {
	fn new() {}

	fn get(&self, name: u32) -> Option<&Box<str>> {
		self.strings.get(name as usize)
	}
}
