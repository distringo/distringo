use core::str::{self};

use tokio::io::AsyncBufReadExt;

use tracing::{error, info, warn};

mod command;
use command::*;

mod mode;
use mode::*;

#[derive(Default)]
pub struct Repl {
	banner_seen: bool,
	history: Vec<String>,
	exiting: bool,
	_mode: Option<ReplMode>,
}

impl Repl {
	async fn banner(&mut self) {
		println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
		println!();

		self.banner_seen = true;
	}

	async fn menu(&mut self) {}

	async fn dispatch(&mut self, command: &ReplCommand) {
		match command {
			ReplCommand::Exit => self.exit().await,
		};
	}

	async fn exit(&mut self) {
		self.exiting = true
	}

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
			Ok(command) => self.dispatch(&command).await,
			Err(error) => warn!("{error}"),
		}
	}

	pub async fn run(&mut self) {
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
				// If no bytes were read, that was EOF (^D).
				if bytes == 0 {
					println!();
					self.exiting = true;
					break;
				}

				// Else, convert to a string.
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

#[cfg(test)]
mod default {
	use super::Repl;

	#[test]
	fn is_expected() {
		// use core::mem::discriminant;

		let Repl {
			banner_seen,
			history,
			exiting,
			// mode,
			..
		} = Repl::default();

		assert_eq!(banner_seen, false);
		assert_eq!(history, Vec::<String>::new());
		assert_eq!(exiting, false);
		// assert_eq!(discriminant(&mode), discriminant(&None));
	}
}
