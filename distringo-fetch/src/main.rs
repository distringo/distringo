#![warn(rust_2018_idioms, future_incompatible)]

use tokio::io::AsyncBufReadExt;

use tracing::{error, info, warn};

#[derive(Default)]
struct ReplContext {
	banner_seen: bool,
	history: Vec<String>,
	exiting: bool,
}

async fn banner(ctx: &mut ReplContext) {
	println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
	println!("");

	ctx.banner_seen = true;
}

async fn menu(ctx: &mut ReplContext) {}

async fn prompt(ctx: &mut ReplContext) {
	use std::io::{stdout, Write};

	if !ctx.banner_seen {
		banner(ctx).await;
	}

	menu(ctx).await;

	print!("> ");

	stdout().flush().expect("error!");
}

async fn command(ctx: &mut ReplContext, input: &str) {
	ctx.history.push(input.to_string());

	match input.to_lowercase().trim() {
		"exit" => ctx.exiting = true,
		command => warn!("unknown command {command}"),
	}
}

async fn repl() {
	let stdin = tokio::io::stdin();
	let mut stdin = tokio::io::BufReader::new(stdin);
	let mut prompt_ctx = ReplContext::default();

	loop {
		// Display a prompt.
		prompt(&mut prompt_ctx).await;

		// Read a line of input.
		let mut buffer = Vec::new();
		let result = stdin.read_until(b'\n', &mut buffer).await;

		match result {
			Ok(0) => {
				print!("\r");
				prompt_ctx.exiting = true;
			}
			Ok(bytes) => {
				// TODO: Potential guardrail for length of input?

				match String::from_utf8(buffer) {
					Ok(input) => command(&mut prompt_ctx, &input).await,
					Err(err) => error!(
						?err,
						"error converting {bytes} of input from utf8, ignoring"
					),
				}
			}
			Err(err) => error!(?err, "error!"),
		}

		if prompt_ctx.exiting {
			break;
		}
	}

	info!("exiting");
}

#[tokio::main]
async fn main() {
	{
		tracing_subscriber::fmt().init();
	}

	if atty::is(atty::Stream::Stdin) && atty::is(atty::Stream::Stdout) {
		repl().await;
	} else {
		println!("not sure what you want me to do here");
	}
}
