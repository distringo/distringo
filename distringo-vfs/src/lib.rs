//! Virtual file system supporting partial resolution.
//!
//! - Maintains a set of roots. By default the roots are only local file directories.
//! - Maintains a lightweight tree structure for each root. The tree structure supports partial resolution, meaning that
//!   it may not be fully resolved yet.
//! - The tree structure is a simple tree of directories and files. Symbolic links are treated as files.
