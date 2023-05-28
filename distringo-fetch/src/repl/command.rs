use core::str::FromStr;

#[derive(Debug)]
pub enum ReplCommand {
	Exit,
}

impl FromStr for ReplCommand {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let lowercased = s.trim().to_lowercase();
		let split: Vec<&str> = lowercased.split_whitespace().collect();

		match &split[..] {
			&["exit"] => Ok(ReplCommand::Exit),
			_ => Err(format!("unknown command {lowercased}")),
		}
	}
}
