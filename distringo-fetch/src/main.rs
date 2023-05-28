#![warn(rust_2018_idioms, future_incompatible)]

use tracing::error;

mod repl;
use repl::*;

#[tokio::main]
async fn main() {
	{
		tracing_subscriber::fmt().init();
	}

	if atty::is(atty::Stream::Stdin) && atty::is(atty::Stream::Stdout) {
		Repl::default().run().await;
	} else {
		error!("not sure what you want me to do here");
	}
}
