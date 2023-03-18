#![warn(rust_2018_idioms, future_incompatible)]

use tokio::io::AsyncBufReadExt;

async fn repl() {
	let stdin = tokio::io::stdin();
	let mut stdin = tokio::io::BufReader::new(stdin);

	loop {
		tracing::info!("Reading");
		// Read a line of input.
		let mut buffer = Vec::new();
		let result = stdin.read_until(b'\n', &mut buffer).await;

		if let Ok(bytes_read) = result {
			tracing::info!("read {bytes_read} bytes");
		} else {
			tracing::info!("error: {:?}", result);
		}
	}
}

#[tokio::main]
async fn main() {
	{
		tracing_subscriber::fmt().init();
	}

	println!("Hello, world!");

	if atty::is(atty::Stream::Stdin) && atty::is(atty::Stream::Stdout) {
		repl().await
	} else {
		println!("not sure what you want me to do here");
	}
}
