use std::collections::HashMap;

struct DatasetConfig {
	identifier: String,
	schema_file: String,
	files: HashMap<String, String>,
}
