#![warn(rust_2018_idioms, future_incompatible)]

use tokio::io::AsyncBufReadExt;

#[derive(Default)]
struct ReplContext {
	banner_seen: bool,
}

async fn menu() {}

async fn prompt(ctx: &mut ReplContext) {
	use std::io::{stdout, Write};

	if !ctx.banner_seen {
		println!("banner");
		ctx.banner_seen = true;
	}

	menu().await;

	print!("> ");

	stdout().flush().expect("error!");
}

async fn repl() {
	let stdin = tokio::io::stdin();
	let mut stdin = tokio::io::BufReader::new(stdin);
	let mut prompt_ctx = ReplContext::default();

	loop {
		tracing::info!("reading input");

		// Display a prompt.
		prompt(&mut prompt_ctx).await;

		// Read a line of input.
		let mut buffer = Vec::new();
		let result = stdin.read_until(b'\n', &mut buffer).await;

		match result {
			Ok(0) => {
				tracing::info!("EOF, bye!");
				break;
			}
			Ok(bytes_read) => {
				tracing::info!("read {bytes_read} bytes");
			}
			Err(err) => tracing::error!(?err, "error!"),
		}
	}
}

#[tokio::main]
async fn main() {
	{
		tracing_subscriber::fmt().init();
	}

	if atty::is(atty::Stream::Stdin) && atty::is(atty::Stream::Stdout) {
		repl().await
	} else {
		println!("not sure what you want me to do here");
	}
}
