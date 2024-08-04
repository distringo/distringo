#![allow(dead_code)]

//! Virtual file system supporting partial resolution and async operations.
//!
//! # Overview
//!
//! A given VFS contains a set of [`VfsNode`]s.
//!
//! ## Nodes
//!
//! A given [`VfsNode`] represents either a directory ([`VfsDirectory`]) or a file ([`VfsFile`]). A given node also has
//! an associated resolution state, which is either Resolved or Unresolved.
//!
//! Resolved nodes have their contents determined, where Unresolved nodes do not. For a directory, resolution involves
//! acquiring a list of directory contents and creating a new node for each entry. For a file, resolution involves reading
//! the contents of the file.
//!
//! ### Roots
//!
//! A given [`Vfs`] will contain a list of roots, which are the starting points for resolution.
//!

pub struct Vfs {
	roots: Vec<VfsNode>,
}

impl Vfs {
	/// Creates a new [`Vfs`] with no roots.
	pub fn empty() -> Self {
		Self { roots: Vec::new() }
	}

	/// Adds a root to the VFS from a given directory.
	pub fn add_root_from_dir<P: AsRef<std::path::Path>>(&mut self, dir: P) {
		let dir_node = VfsDirectory::new(dir);
		self.roots.push(VfsNode::Directory(dir_node));
	}
}

pub enum VfsNode {
	Directory(VfsDirectory),
	File(VfsFile),
}

impl Resolvable for VfsNode {
	fn resolve(&mut self) {
		match self {
			VfsNode::Directory(dir) => dir.resolve(),
			VfsNode::File(file) => file.resolve(),
		}
	}
}

impl AsyncResolvable for VfsNode {
	async fn resolve_async(&mut self) {
		match self {
			VfsNode::Directory(dir) => dir.resolve_async().await,
			VfsNode::File(file) => file.resolve_async().await,
		}
	}
}

trait AsyncResolvable {
	async fn resolve_async(&mut self);
}

trait Resolvable {
	fn resolve(&mut self);
}

pub struct VfsDirectory {
	dirname: std::path::PathBuf,
	contents: Option<Vec<VfsNode>>,
}

impl AsyncResolvable for VfsDirectory {
	async fn resolve_async(&mut self) {
		todo!()
	}
}

impl Resolvable for VfsDirectory {
	fn resolve(&mut self) {
		todo!()
	}
}

impl VfsDirectory {
	pub fn new<P: AsRef<std::path::Path>>(dirname: P) -> Self {
		Self {
			dirname: dirname.as_ref().to_path_buf(),
			contents: None,
		}
	}
}

pub struct VfsFile;

impl Resolvable for VfsFile {
	fn resolve(&mut self) {
		todo!()
	}
}

impl AsyncResolvable for VfsFile {
	async fn resolve_async(&mut self) {
		todo!()
	}
}
