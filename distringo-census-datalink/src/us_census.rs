use std::{collections::VecDeque, fs::File, io::Write, thread::sleep};

pub async fn get_recursive_directory_listing() -> impl core::fmt::Debug {
	println!("connecting...");

	let mut ftp_stream = suppaftp::FtpStream::connect("ftp2.census.gov:21")
		.unwrap_or_else(|err| panic!("no connection... {err}"))
		.active_mode();

	println!("logging in");

	ftp_stream
		.login("anonymous", "")
		.expect("failed to log in!");

	println!("checking welcome message");

	// Gut-check the welcome message.
	match ftp_stream.get_welcome_msg() {
		Some("220") => {}
		_ => panic!("unexpected welcome message"),
	}

	let mut queue: VecDeque<Vec<String>> = VecDeque::from([vec![
		"programs-surveys".to_string(),
		"decennial".to_string(),
		"2020".to_string(),
		"data".to_string(),
		"01-Redistricting_File--PL_94-171".to_string(),
	]]);

	let mut file = File::create("./listing.txt").expect("failed to open listing file");

	while let Some(next) = queue.pop_front() {
		println!("sleeping temporarily...");

		sleep(core::time::Duration::from_millis(100));

		println!("woke up...");

		let mut string: String = String::default();

		let cur_dir = match next.len() {
			0 => None,
			_ => Some(next.join("/")),
		};

		println!("scanning dir {:?}", next);

		let listing = match ftp_stream.list(cur_dir.as_deref()) {
			Ok(listing) => listing,
			Err(asdf) => {
				eprintln!("error: {asdf}; placing back on queue");
				queue.push_back(next);
				continue;
			}
		};

		// let listing = ftp_stream.list(cur_dir.as_deref()).expect(&format!(
		// 	"expected good result when requesting listing for next dir {next:?}"
		// ));

		println!("got listing of {} entries", listing.len());

		let mut files: Vec<String> = Vec::with_capacity(listing.len());
		let mut directories: Vec<String> = Vec::with_capacity(listing.len());
		let mut others: Vec<String> = Vec::with_capacity(listing.len());

		for line in listing {
			println!("{line}");

			let mut split = line.split_ascii_whitespace();

			let file_mode: Vec<u8> = split
				.next()
				.expect("expected file mode part")
				.bytes()
				.collect();

			let (ty, _own_pex, _grp_pex, _oth_pex, _rest) = {
				let ty: &[u8] = &file_mode[0..1];
				let own_pex: &[u8] = &file_mode[1..4];
				let grp_pex: &[u8] = &file_mode[4..7];
				let oth_pex: &[u8] = &file_mode[7..10];
				let rest: &[u8] = &file_mode[10..];
				(ty, own_pex, grp_pex, oth_pex, rest)
			};

			let _num_links = split.next().expect("expected directory type part");
			let _owner_name = split.next().expect("expected directory type part");
			let _group_name = split.next().expect("expected directory type part");
			let _num_bytes = split.next().expect("expected directory type part");
			let _last_mod_mo = split.next().expect("expected directory type part");
			let _last_mod_da = split.next().expect("expected directory type part");
			let _last_mod_yrtm = split.next().expect("expected directory type part");

			let name: Vec<&str> = split.collect();

			let name = if name.is_empty() {
				"".to_string()
			} else if name.len() == 1 {
				name[0].to_string()
			} else if let Some(pos) = name.iter().position(|&s| s == "->") {
				name[0..pos].join(" ")
			} else {
				name[0..].join(" ")
			};

			#[derive(PartialEq)]
			enum EntryType {
				Directory,
				File,
				Other,
			}

			let entry_type = match *ty {
				[b'd'] => EntryType::Directory,
				[b'-'] => EntryType::File,
				_ => EntryType::Other,
			};

			match entry_type {
				EntryType::Directory => directories.push(name.to_string()),
				EntryType::File => files.push(name.to_string()),
				EntryType::Other => {
					println!("skipping scan of special listing {line}");
					others.push(name.to_string());
					continue;
				}
			}
		}

		// dbg!(&files, &directories, &others);

		string = format!("{string}{}:", cur_dir.expect("expected dirname"));

		for directory in directories {
			string = format!("{string}\n{}", directory);

			let mut path = next.clone();
			path.push(directory);
			queue.push_back(path);
		}

		for file in files {
			string = format!("{string}\n{}", file);
		}

		writeln!(file, "{string}\n").expect("foo");
	}

	//
}
