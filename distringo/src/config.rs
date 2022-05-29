use std::collections::HashMap;

struct Dataset {
	id: String,
	schema: String,
	files: HashMap<String, String>,
}

struct Shapefile {
	id: String,
	file: String,
}
