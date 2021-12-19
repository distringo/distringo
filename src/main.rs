pub mod server;

/// Get the settings from defaults, the environment, and the config file.
fn get_settings() -> core::result::Result<config::Config, config::ConfigError> {
	use config::{Config, Environment, File};

	let mut settings = Config::default();

	settings.set_default("server.host", "::")?;
	settings.set_default("server.port", 2020)?;

	settings.merge(Environment::with_prefix("DISTRINGO"))?;

	settings.merge(File::with_name("config"))?;

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

	#[error("invalid server host")]
	InvalidServerHost,
	#[error("invalid server port")]
	InvalidServerPort,
}

type Result<T, E = RuntimeError> = core::result::Result<T, E>;

#[tokio::main]
async fn main() -> Result<()> {
	if std::env::var("DISTRINGO_LOG").ok().is_none() {
		std::env::set_var("DISTRINGO_LOG", "info");
	}

	pretty_env_logger::init_custom_env("DISTRINGO_LOG");

	let settings = get_settings()?;

	let mut plan: server::ExecutionPlan = server::ExecutionPlan::try_from(settings)?;
	plan.prepare()?;
	plan.execute().await
}
