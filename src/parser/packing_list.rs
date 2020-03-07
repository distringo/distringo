use crate::error::Result;
use core::fmt::Display;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use log::debug;
use regex::Regex;

#[derive(Debug)]
struct SegmentationInformation {
	table: String,
	file_width: Vec<(u16, usize)>,
}

#[derive(Clone, Debug, PartialEq)]
enum FileType {
	HeaderFile(String),
	TabularFile(u16),
}

#[derive(Clone, Debug)]
struct DatasetFile {
	filename: String,
	extension: String,
	stusab: String,
	descriptor: String,
	year: String,
	ty: FileType,
}

impl DatasetFile {
	fn is_tabular(&self) -> bool {
		match self.ty {
			FileType::TabularFile(_) => true,
			_ => false,
		}
	}

	fn schema(&self) -> crate::schema::CensusDataSchema {
		match (&self.year[..], &self.extension[..]) {
			("2010", "pl") => crate::schema::CensusDataSchema::Census2010Pl94_171,
			(_, _) => unimplemented!(),
		}
	}
}

#[derive(Debug)]
pub struct PackingList {
	schema: crate::schema::CensusDataSchema,
	files: Vec<DatasetFile>,
	tables: BTreeMap<String, SegmentationInformation>,
}

impl PackingList {
	pub fn schema(&self) -> crate::schema::CensusDataSchema {
		self.schema
	}
}

use lazy_static::lazy_static;

lazy_static! {
	static ref SEGMENTATION_INFORMATION: Regex =
		Regex::new("^(?P<table>[a-z0-9]+)\\|(?P<descriptor>[\\d: ]*)\\|$").unwrap();
	static ref FILE_INFORMATION: Regex =
		Regex::new("^(?P<filename>.*)\\|(?P<date>\\d{4}-\\d{2}-\\d{2} \\d{2}:\\d{2}:\\d{2}\\|(?P<size>\\d+)\\|(?P<rows>\\d+)\\|$)").unwrap();
	static ref FILE_NAME: Regex =
		Regex::new("^(?P<stusab>[a-z]{2})(?P<descriptor>.*)(?P<year>\\d{4})\\.(?P<type>.*)$").unwrap();
}

enum Line {
	DataSegmentation(SegmentationInformation),
	FileInformation(DatasetFile),
	None,
}

impl core::convert::TryFrom<String> for SegmentationInformation {
	type Error = crate::error::Error;

	fn try_from(line: String) -> crate::error::Result<Self> {
		let caps = SEGMENTATION_INFORMATION
			.captures(&line)
			.ok_or(crate::error::Error::ParsePackingListLine)?;

		let file_width = caps["descriptor"]
			.split(" ")
			.map(|chunk: &str| -> Result<(u16, usize)> {
				let file: u16 = chunk.split(":").collect::<Vec<&str>>()[0].parse()?;
				let width: usize = chunk.split(":").collect::<Vec<&str>>()[1].parse()?;
				Ok((file, width))
			})
			.filter_map(Result::ok)
			.collect();

		Ok(SegmentationInformation {
			table: caps["table"].to_string(),
			file_width,
		})
	}
}

impl core::convert::TryFrom<String> for DatasetFile {
	type Error = crate::error::Error;

	fn try_from(line: String) -> crate::error::Result<Self> {
		let caps = FILE_INFORMATION
			.captures(&line)
			.ok_or(crate::error::Error::ParsePackingListLine)?;

		let filename: String = caps["filename"].to_string();

		let filename_caps = FILE_NAME
			.captures(&filename)
			.ok_or(crate::error::Error::ParsePackingListFilename)?;

		let stusab: String = filename_caps["stusab"].to_string();
		let descriptor: String = filename_caps["descriptor"].to_string();
		let year: String = filename_caps["year"].to_string();
		let extension: String = filename_caps["type"].to_string();

		let ty: FileType = match descriptor.parse::<u16>() {
			Ok(index) => FileType::TabularFile(index),
			Err(_) => FileType::HeaderFile(descriptor.clone()),
		};

		Ok(DatasetFile {
			filename,
			descriptor,
			ty,
			year,
			stusab,
			extension,
		})
	}
}

impl From<String> for Line {
	fn from(line: String) -> Self {
		use core::convert::TryInto;
		if SEGMENTATION_INFORMATION.is_match(&line) {
			debug!("Interpreting \"{}\" as Data Segmentation", line);
			line
				.try_into()
				.map(Self::DataSegmentation)
				.unwrap_or(Self::None)
		} else if FILE_INFORMATION.is_match(&line) {
			debug!("Interpreting line \"{}\" as File Information", line);
			line
				.try_into()
				.map(Self::FileInformation)
				.unwrap_or(Self::None)
		} else {
			Self::None
		}
	}
}

impl PackingList {
	pub fn from_file<P: AsRef<Path> + Display + Sized>(file: P) -> Result<PackingList> {
		debug!("Loading packing list from {}", file);

		let file: File = File::open(file)?;
		let stream = BufReader::new(file);

		let lines: Vec<Line> = stream
			.lines()
			.filter_map(std::io::Result::ok)
			.map(Into::into)
			.collect();

		let files: Vec<DatasetFile> = {
			lines
				.iter()
				.filter_map(|line: &Line| -> Option<&DatasetFile> {
					match line {
						Line::FileInformation(file) => Some(&file),
						_ => None,
					}
				})
				.cloned()
				.collect()
		};

		let schemas: std::collections::BTreeSet<crate::schema::CensusDataSchema> = files
			.iter()
			.map(|file: &DatasetFile| -> crate::schema::CensusDataSchema { file.schema() })
			.collect();

		debug_assert!(schemas.len() == 1);

		let schema: crate::schema::CensusDataSchema = schemas
			.iter()
			.nth(0)
			.expect("couldn't infer a schema")
			.clone();

		let tables: BTreeMap<String, SegmentationInformation> = BTreeMap::new();

		Ok(PackingList {
			schema,
			files,
			tables,
		})
	}
}
