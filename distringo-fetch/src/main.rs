#![warn(rust_2018_idioms, future_incompatible)]

use std::io::{stdin, stdout, IsTerminal};

use tracing::error;

mod repl;
use repl::*;

#[tokio::main]
async fn main() {
	{
		tracing_subscriber::fmt().init();
	}

	if stdin().is_terminal() && stdout().is_terminal() {
		Repl::default().run().await;
	} else {
		error!("not sure what you want me to do here");
	}
}
