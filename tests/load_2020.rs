use distringo::Dataset;

use fnv::FnvHashMap;

/// Simple loading example
///
/// Reads a packing list from
#[test]
fn main() -> distringo::Result<()> {
	const TABLE_MAP: [distringo::Table; 6] = {
		use distringo::{census2020::pl94_171::*, census2020::Table::Pl94_171, Table::Census2020};
		[
			Census2020(Pl94_171(P1)),
			Census2020(Pl94_171(P2)),
			Census2020(Pl94_171(P3)),
			Census2020(Pl94_171(P4)),
			Census2020(Pl94_171(H1)),
			Census2020(Pl94_171(P5)),
		]
	};

	simple_logger::SimpleLogger::new()
		.with_level(log::LevelFilter::Trace)
		.with_utc_timestamps()
		.init()
		.unwrap();

	let schema = distringo::Schema::Census2020(distringo::census2020::Schema::Pl94_171);

	let directory = Some(
		std::path::PathBuf::from(file!())
			.parent()
			.expect("what")
			.join("data"),
	);

	let table_locations = TABLE_MAP.iter().map(|&table| (table, vec![])).collect();

	let tabular_files = {
		let mut map = FnvHashMap::default();

		map.insert(
			0_u32,
			std::path::PathBuf::from("ri000012018_2020Style.pl.trim"),
		);
		map.insert(
			1_u32,
			std::path::PathBuf::from("ri000022018_2020Style.pl.trim"),
		);
		map.insert(
			2_u32,
			std::path::PathBuf::from("ri000032018_2020Style.pl.trim"),
		);

		map
	};

	let geographical_header_file = std::path::PathBuf::from("rigeo2018_2020Style.pl.trim");
	let rows = 19346_usize;

	let pl: distringo::PackingList = distringo::PackingList::new(
		schema,
		directory,
		table_locations,
		tabular_files,
		geographical_header_file,
		rows,
	);

	let ds = distringo::IndexedDataset::from_packing_list(pl)?.index()?;

	let logrecno: distringo::LogicalRecordNumber = 19_200;

	let record = ds.get_logical_record(logrecno)?;

	let rec_a: Vec<&str> = vec![
		"PLST", "RI", "000", "02", "0019200", "24", "24", "24", "0", "0", "0", "0", "0", "0", "0", "0",
		"0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
		"0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
		"0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
		"0", "0", "0", "24", "0", "24", "24", "24", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
		"0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
		"0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
		"0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
		"0", "26", "25", "1",
	];

	let rec_b: Vec<&str> = vec![
		"PLST", "RI", "000", "01", "0019200", "25", "25", "25", "0", "0", "0", "0", "0", "0", "0", "0",
		"0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
		"0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
		"0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
		"0", "0", "0", "25", "1", "24", "24", "24", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
		"0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
		"0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
		"0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
		"0",
	];

	let rec_c: Vec<&str> = vec![
		"PLST", "RI", "000", "03", "0019200", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
	];

	let rec_a: csv::StringRecord = rec_a.into();
	let rec_b: csv::StringRecord = rec_b.into();
	let rec_c: csv::StringRecord = rec_c.into();

	assert_eq!(
		record
			.raw_records()
			.values()
			.collect::<Vec<&csv::StringRecord>>(),
		vec![&rec_a, &rec_b, &rec_c]
	);

	let logrecno = ds.get_logical_record_number_for_geoid("440070184001012")?;
	assert_eq!(logrecno, 19_200);

	let header = ds.get_header_for_geoid("440070184001012")?;
	assert_eq!(header.name(), "Block 1012");
	assert_eq!(header.logrecno(), 19_200);

	Ok(())
}