use std::path::PathBuf;

/// Returns the directory containing this file.
fn this_dir() -> PathBuf {
	let this_file = file!();
	let this_dir = PathBuf::from(this_file);
	this_dir
		.parent()
		.expect("failed to get parent of this file")
		.into()
}

fn main() {
	let _root_dir = this_dir();
}
