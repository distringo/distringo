#![deny(clippy::all)]
#![warn(clippy::cargo)]
#![warn(clippy::pedantic)]
#![warn(clippy::cognitive_complexity)]

pub mod server;

/// Get the settings from defaults, the environment, and the config file.
fn get_settings() -> core::result::Result<config::Config, config::ConfigError> {
	use config::{builder::DefaultState, Config, ConfigBuilder, Environment, File};

	let settings: Config = ConfigBuilder::<DefaultState>::default()
		.set_default("version", env!("CARGO_PKG_VERSION"))?
		.set_default("server.host", "::")?
		.set_default("server.port", 2020_u16)?
		.add_source(Environment::with_prefix("DISTRINGO"))
		.add_source(File::with_name("config"))
		.build()?;

	// TODO(rye): Partial loading of subkeys from associated files.
	//
	// E.g. Where we have a `datasets` key at the top level in config.yml, allow for merging (on top
	// of what is already there) files from the `config/datasets/` folder. So, for example,
	// `config/datasets/asdf.yml` would get imported under the configuration's `datasets.asdf` key.

	Ok(settings)
}

#[derive(thiserror::Error, Debug)]
pub enum RuntimeError {
	#[error("configuration error")]
	Config(#[from] config::ConfigError),
	#[error("geojson error")]
	GeoJson(#[from] geojson::Error),
	#[error("i/o error")]
	Io(#[from] std::io::Error),

	#[error("invalid configuration")]
	Configuration(#[from] server::AppConfigError),

	#[error("web server internal error")]
	Hyper(#[from] hyper::Error),

	#[error("invalid server host")]
	InvalidServerHost,
	#[error("invalid server port")]
	InvalidServerPort,
}

type Result<T, E = RuntimeError> = core::result::Result<T, E>;

#[tokio::main]
async fn main() -> Result<()> {
	// Set up logging.
	{
		use tracing_subscriber::filter::EnvFilter;

		let filter = EnvFilter::from_env("DISTRINGO_LOG");
		tracing_subscriber::fmt().with_env_filter(filter).init();
	}

	let settings = get_settings()?;

	let mut plan: server::ExecutionPlan = server::ExecutionPlan::from(settings);
	plan.validate().await?;
	plan.prepare()?;
	plan.execute().await
}
