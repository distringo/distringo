use axum::{error_handling::HandleErrorLayer, routing::get};
use http::StatusCode;
use tower::BoxError;
use tower_http::trace::TraceLayer;

use crate::Result;

mod routes;
use routes::*;

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

		// for (identifier, value) in config.get_table("datasets")? {
		// 	println!("{:?}, {:?}", identifier, value);
		// }

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

		log::trace!("Executing Execution Plan");

		let config: &config::Config = &self.0;

		let socket = {
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

		tracing::debug!("socket: {:?}", socket);

		let router = routes::app_router(config)?;

		axum::Server::bind(&socket)
			.serve(router.into_make_service())
			.await?;

		Ok(())
	}
}
