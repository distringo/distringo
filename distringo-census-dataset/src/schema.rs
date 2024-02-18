#![allow(unused_variables, dead_code)]

use std::{collections::HashSet, ops::RangeInclusive, path::Path};

use tokio::io::AsyncBufReadExt;

struct CensusDatasetSchema {
	pub name: String,
	pub geo_header_fields: Vec<CensusDataFieldSpec>,
	pub tables: Vec<CensusDataTableSpec>,
	pub delim: Option<char>,
}

enum CensusDataFieldType {
	AlphaNumeric,
	Alpha,
	Numeric,
	Integer,
	Decimal,
}

enum CensusDataFieldSpec {
	Unspecified(&'static str),
	Specified {
		field_name: String,
		dictionary_name: String,
		data_type: CensusDataFieldType,
		size: RangeInclusive<usize>,
		summary_levels: CensusSummaryLevelRequirement,
	},
}

enum CensusSummaryLevelRequirement {
	AllLevels,
	SomeLevels(HashSet<CensusDataSummaryLevel>),
	Unspecified,
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct CensusDataSummaryLevel {
	pub number: String,
}

impl From<&str> for CensusDataSummaryLevel {
	fn from(s: &str) -> Self {
		Self {
			number: s.to_string(),
		}
	}
}

struct CensusDataTableSpec {
	pub name: String,
	pub fields: Vec<CensusDataFieldSpec>,
}

impl CensusDatasetSchema {
	pub async fn check_files(
		&self,
		geo_header: impl AsRef<Path>,
		table_files: Vec<impl AsRef<Path>>,
	) -> std::io::Result<bool> {
		let geo_header = geo_header.as_ref();

		// Stage 1: Check the geographic header file.
		let geo_header = tokio::fs::File::open(geo_header).await.unwrap();
		let file = tokio::io::BufReader::new(geo_header);

		let mut line_stream = file.lines();

		let mut geo_header_ok: bool = true;

		while let Some(line) = line_stream.next_line().await? {
			let fields: Vec<&str> = match self.delim {
				Some(delim) => line.split(delim).collect(),
				None => todo!("support for fixed-width files"),
			};

			if self.geo_header_fields.len() != fields.len() {
				println!(
					"Wrong number of fields: {} (expected {})",
					fields.len(),
					self.geo_header_fields.len()
				);

				geo_header_ok = false;
				break;
			}

			let row_fileid = fields[0];
			let row_stusab = fields[1];
			let row_sumlev = fields[2].into();
			let row_geovar = fields[3];
			let row_geocomp = fields[4];
			let row_chariter = fields[5];
			let row_cifsn = fields[6];
			let row_logrecno = fields[7];

			for (i, field_data) in fields.iter().enumerate() {
				let spec = match self.geo_header_fields.get(i) {
					Some(spec) => spec,
					None => {
						println!("Extra field data: {}", field_data);
						geo_header_ok = false;
						break;
					}
				};

				match spec {
					// With unspecified fields, we can't check anything.
					CensusDataFieldSpec::Unspecified(_) => {}

					// Otherwise, we can check the field specification.
					CensusDataFieldSpec::Specified {
						field_name,
						dictionary_name,
						data_type,
						size,
						summary_levels,
					} => {
						let field_data = field_data;

						// Check the size...
						if field_data.len() < *size.start() || field_data.len() > *size.end() {
							println!(
								"Field {} is the wrong size: {} (expected {}-{})",
								field_name,
								field_data.len(),
								size.start(),
								size.end()
							);
							geo_header_ok = false;
							break;
						}

						// Check the data type...
						match data_type {
							CensusDataFieldType::AlphaNumeric => {
								if !field_data.chars().all(|c| c.is_ascii_alphanumeric()) {
									println!("Field {} is not alphanumeric: {}", field_name, field_data);
									geo_header_ok = false;
									break;
								}
							}
							CensusDataFieldType::Alpha => {
								if !field_data.chars().all(|c| c.is_ascii_alphabetic()) {
									println!("Field {} is not alphabetic: {}", field_name, field_data);
									geo_header_ok = false;
									break;
								}
							}
							CensusDataFieldType::Numeric => {
								if !field_data.chars().all(|c| c.is_ascii_digit()) {
									println!("Field {} is not numeric: {}", field_name, field_data);
									geo_header_ok = false;
									break;
								}
							}
							CensusDataFieldType::Integer => {
								if !field_data.chars().all(|c| c.is_ascii_digit()) {
									println!("Field {} is not an integer: {}", field_name, field_data);
									geo_header_ok = false;
									break;
								}
							}
							CensusDataFieldType::Decimal => {
								if !field_data.chars().all(|c| c.is_ascii_digit()) {
									println!("Field {} is not a decimal: {}", field_name, field_data);
									geo_header_ok = false;
									break;
								}
							}
						}

						// Check the summary levels...
						match summary_levels {
							CensusSummaryLevelRequirement::AllLevels => {
								if field_data.is_empty() {
									println!(
										"Field {} is empty, but it's required at all levels.",
										field_name
									);
									geo_header_ok = false;
									break;
								}
							}
							CensusSummaryLevelRequirement::SomeLevels(levels) => {
								if !field_data.is_empty() && !levels.contains(&row_sumlev) {
									println!(
										"Field {} appears at an unexpected level: {:?}.",
										field_name, row_sumlev
									);
									geo_header_ok = false;
									break;
								}
							}
							CensusSummaryLevelRequirement::Unspecified => {
								// We can't check anything.
							}
						}
					}
				}
			}
		}

		// Stage 2: Check the tables.

		Ok(geo_header_ok)
	}
}
