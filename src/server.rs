use crate::Result;

pub mod routes;

async fn handle_rejection(
	err: warp::Rejection,
) -> core::result::Result<impl warp::Reply, core::convert::Infallible> {
	if err.is_not_found() {
		Ok(warp::reply::with_status(
			warp::reply::html(include_str!("404.html")),
			http::StatusCode::NOT_FOUND,
		))
	} else {
		log::warn!("unhandled rejection: {:?}", err);
		Ok(warp::reply::with_status(
			warp::reply::html(include_str!("500.html")),
			http::StatusCode::INTERNAL_SERVER_ERROR,
		))
	}
}
pub struct ExecutionPlan(config::Config);

#[derive(thiserror::Error, Debug)]
pub enum AppConfigError {
	#[error("inner configuration error")]
	Config(#[from] config::ConfigError),
	#[error("version parse error")]
	Semver(#[from] semver::Error),

	#[error("configuration version does not meet requirements")]
	InvalidVersion,
}

impl From<config::Config> for ExecutionPlan {
	fn from(config: config::Config) -> Self {
		Self(config)
	}
}

impl ExecutionPlan {
	fn validate_version(version: &str) -> Result<(), AppConfigError> {
		const VERSION_REQUIREMENT: &str = "~0.0.0";

		let requirement = semver::VersionReq::parse(VERSION_REQUIREMENT)
			.expect("internally-generated version requirement was invalid");

		let version: semver::Version = version.parse()?;

		if !requirement.matches(&version) {
			Err(AppConfigError::InvalidVersion)
		} else {
			Ok(())
		}
	}

	pub async fn validate(&self) -> Result<(), AppConfigError> {
		let config = &self.0;

		// Verify that the version is valid.
		let config_version = config.get_str("version")?;
		Self::validate_version(&config_version)?;

		// Load up all the dataset configurations.

		for (identifier, value) in config.get_table("datasets")? {
			println!("{:?}, {:?}", identifier, value);
		}

		Ok(())
	}

	pub fn prepare(&self) -> Result<()> {
		// TODO(rye): Instead of doing nothing, perform a dry run of creating the
		// routes here so as to early-die if something is amiss.
		//
		// (Ideally, reduce the contract of execute())

		Ok(())
	}

	pub async fn execute(&mut self) -> Result<()> {
		use std::net::{IpAddr, SocketAddr};

		let socket = {
			let config: &config::Config = &self.0;

			let host: IpAddr = config
				.get_str("server.host")?
				.parse()
				.map_err(|_| crate::RuntimeError::InvalidServerHost)?;
			let port: u16 = config
				.get_int("server.port")?
				.try_into()
				.map_err(|_| crate::RuntimeError::InvalidServerPort)?;

			SocketAddr::new(host, port)
		};

		warp::serve(routes::routes(&self.0)?).run(socket).await;

		Ok(())
	}
}
