#![warn(rust_2018_idioms, future_incompatible)]

use tokio::io::AsyncBufReadExt;

use tracing::{error, info, warn};

#[derive(Default)]
struct Repl {
	banner_seen: bool,
	history: Vec<String>,
	exiting: bool,
}

impl Repl {
	async fn banner(&mut self) {
		println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
		println!("");

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

		match input.to_lowercase().trim() {
			"exit" => self.exiting = true,
			command => warn!("unknown command {command}"),
		}
	}

	async fn run(&mut self) {
		let stdin = tokio::io::stdin();
		let mut stdin = tokio::io::BufReader::new(stdin);

		loop {
			// Display a prompt.
			self.prompt().await;

			// Read a line of input.
			let mut buffer = Vec::new();
			let result = stdin.read_until(b'\n', &mut buffer).await;

			match result {
				Ok(0) => {
					print!("\r");
					self.exiting = true;
				}
				Ok(bytes) => {
					// TODO: Potential guardrail for length of input?

					match String::from_utf8(buffer) {
						Ok(input) => self.command(&input).await,
						Err(err) => error!(
							?err,
							"error converting {bytes} of input from utf8, ignoring"
						),
					}
				}
				Err(err) => error!(?err, "error!"),
			}

			if self.exiting {
				break;
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
