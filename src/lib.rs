use fnv::FnvHashMap;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub mod error;

pub type LogicalRecordNumber = u64;
pub type GeoId = String;

pub(crate) struct LogicalRecordPositionIndex {
	inner: Vec<Option<u64>>,
}

impl LogicalRecordPositionIndex {
	fn new_with_size(size: usize) -> Self {
		Self {
			inner: Vec::with_capacity(size),
		}
	}

	fn insert(&mut self, logrecno: LogicalRecordNumber, offset: u64) {
		let idx: usize = logrecno as usize;
		self.inner.resize(idx + 1, None);
		self.inner[idx] = Some(offset);
	}
}

impl core::ops::Index<LogicalRecordNumber> for LogicalRecordPositionIndex {
	type Output = Option<u64>;
	fn index(&self, logrecno: LogicalRecordNumber) -> &Option<u64> {
		&self.inner[logrecno as usize]
	}
}

/// A Logical Record
pub trait LogicalRecord {
	/// Get the corresponding number
	///
	/// The Census refers to records by their "logical number."  A logical record
	/// is assumed _only_ to have this number.
	fn number(&self) -> LogicalRecordNumber;
}

mod dataset;
pub use dataset::*;

pub struct FileBackedLogicalRecord {
	number: LogicalRecordNumber,
	records: FnvHashMap<usize, csv::StringRecord>,
}

impl LogicalRecord for FileBackedLogicalRecord {
	fn number(&self) -> LogicalRecordNumber {
		self.number
	}
}

impl FileBackedLogicalRecord {
	fn new(number: LogicalRecordNumber) -> Self {
		Self {
			number,

			records: FnvHashMap::default(),
		}
	}

	fn records(mut self, records: BTreeMap<usize, csv::StringRecord>) -> Self {
		self.records.extend(records);
		self
	}
}

/// A geographical header
pub trait GeographicalHeader {
	fn name(&self) -> &str;
	fn logrecno(&self) -> LogicalRecordNumber;
}

pub mod census2010;

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Schema {
	Census2010Pl94_171(Option<census2010::pl94_171::Table>),
}

impl<S: AsRef<str>> core::convert::From<S> for Schema {
	fn from(s: S) -> Self {
		let s: &str = s.as_ref();
		match s {
			"p1" => Schema::Census2010Pl94_171(Some(census2010::pl94_171::P1)),
			"p2" => Schema::Census2010Pl94_171(Some(census2010::pl94_171::P2)),
			"p3" => Schema::Census2010Pl94_171(Some(census2010::pl94_171::P3)),
			"p4" => Schema::Census2010Pl94_171(Some(census2010::pl94_171::P4)),
			"h1" => Schema::Census2010Pl94_171(Some(census2010::pl94_171::H1)),
			_ => unimplemented!(),
		}
	}
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub(crate) enum FileType {
	Census2010Pl94_171(census2010::pl94_171::FileType),
}

impl FileType {
	fn is_header(&self) -> bool {
		match self {
			Self::Census2010Pl94_171(census2010::pl94_171::FileType::GeographicalHeader) => true,
			_ => false,
		}
	}

	fn is_tabular(&self) -> bool {
		match self {
			Self::Census2010Pl94_171(census2010::pl94_171::FileType::Tabular(_)) => true,
			_ => false,
		}
	}

	fn tabular_index(&self) -> Option<usize> {
		match self {
			Self::Census2010Pl94_171(census2010::pl94_171::FileType::Tabular(n)) => Some(*n),
			_ => None,
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::census2010::pl94_171::Table;
	use crate::Schema;

	#[test]
	fn schema_with_table_de() {
		let data = r"Census2010Pl94_171: P1";
		let schema: Schema = serde_yaml::from_str(data).unwrap();
		assert_eq!(schema, Schema::Census2010Pl94_171(Some(Table::P1)))
	}

	#[test]
	fn bare_schema_de() {
		let data = r"Census2010Pl94_171:";
		let schema: Schema = serde_yaml::from_str(data).unwrap();
		assert_eq!(schema, Schema::Census2010Pl94_171(None))
	}
}

pub(crate) type GeographicalHeaderIndex = BTreeMap<GeoId, (LogicalRecordNumber, u64)>;
pub(crate) type LogicalRecordIndex = FnvHashMap<FileType, LogicalRecordPositionIndex>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TableSegmentSpecifier {
	file: usize,
	columns: usize,
}

#[derive(Clone, Debug)]
pub struct TableSegmentLocation {
	file: usize,
	range: core::ops::Range<usize>,
}

pub type TableName = String;
pub type TableLocationSpecifier = Vec<TableSegmentSpecifier>;
pub type TableLocations = Vec<TableSegmentLocation>;
