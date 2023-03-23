#![warn(rust_2018_idioms, future_incompatible)]

use core::str::{self, FromStr};

use tokio::io::AsyncBufReadExt;

use tracing::{error, info, warn};

#[derive(Default)]
struct Repl {
	banner_seen: bool,
	history: Vec<String>,
	exiting: bool,
}

#[derive(Debug)]
enum ReplCommand {
	Exit,
}

impl FromStr for ReplCommand {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let lowercased = s.trim().to_lowercase();
		let split: Vec<&str> = lowercased.split_whitespace().collect();

		match &split[..] {
			&["exit"] => Ok(ReplCommand::Exit),
			_ => Err(format!("unknown input {lowercased}")),
		}
	}
}

impl Repl {
	async fn banner(&mut self) {
		println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
		println!();

		self.banner_seen = true;
	}

	async fn menu(&mut self) {}

	async fn prompt(&mut self) {
		use std::io::{stdout, Write};

		if !self.banner_seen {
			self.banner().await;
		}

		self.menu().await;

		print!("> ");

		stdout().flush().expect("error!");
	}

	async fn command(&mut self, input: &str) {
		self.history.push(input.to_string());

		match input.parse() {
			Ok(ReplCommand::Exit) => self.exiting = true,
			Err(error) => warn!("{error}"),
		}
	}

	async fn run(&mut self) {
		let stdin = tokio::io::stdin();
		let mut stdin = tokio::io::BufReader::new(stdin);
		let mut buffer = Vec::new();

		while !self.exiting {
			// Clear the input buffer.
			buffer.clear();

			// Display a prompt.
			self.prompt().await;

			// Read a line of input.
			let result = stdin.read_until(b'\n', &mut buffer).await;

			// Check the result and execute a command if possible.
			if let Ok(bytes) = result {
				if bytes == 0 {
					print!("\r");
					self.exiting = true;
					break;
				}

				match str::from_utf8(&buffer) {
					Ok(input) => self.command(input).await,
					Err(err) => error!(
						?err,
						"error converting {bytes} of input from utf8, ignoring"
					),
				}
			} else if let Err(err) = result {
				error!(?err, "error!");
			}
		}

		info!("exiting");
	}
}

#[tokio::main]
async fn main() {
	{
		tracing_subscriber::fmt().init();
	}

	if atty::is(atty::Stream::Stdin) && atty::is(atty::Stream::Stdout) {
		Repl::default().run().await;
	} else {
		println!("not sure what you want me to do here");
	}
}
