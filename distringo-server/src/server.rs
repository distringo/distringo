use std::net::SocketAddr;

use crate::{settings::AppConfig, Result};

mod routes;

#[derive(thiserror::Error, Debug)]
pub enum AppConfigError {
	#[error("inner configuration error")]
	Config(#[from] config::ConfigError),
	#[error("version parse error")]
	Semver(#[from] semver::Error),

	#[error("configuration version does not meet requirements")]
	InvalidVersion,
}

pub struct ExecutionPlan {
	config: AppConfig,
}

impl From<AppConfig> for ExecutionPlan {
	fn from(config: AppConfig) -> Self {
		Self { config }
	}
}

const CONFIG_VERSION_REQUIREMENT: &str = "~0.0.0";

impl ExecutionPlan {
	fn validate_version(
		version: &Option<semver::Version>,
		requirement: semver::VersionReq,
	) -> Result<(), AppConfigError> {
		if let Some(version) = version {
			if requirement.matches(version) {
				Ok(())
			} else {
				Err(AppConfigError::InvalidVersion)
			}
		} else {
			Err(AppConfigError::InvalidVersion)
		}
	}

	pub async fn validate(&self) -> Result<(), AppConfigError> {
		let config = &self.config;

		// Verify that the version of the configuration is valid.
		Self::validate_version(config.version(), CONFIG_VERSION_REQUIREMENT.parse()?)?;

		Ok(())
	}

	pub fn prepare(&self) -> Result<()> {
		// TODO(rye): Instead of doing nothing, perform a dry run of creating the
		// routes here so as to early-die if something is amiss.
		//
		// (Ideally, reduce the contract of execute())

		Ok(())
	}

	fn bind_addr(&self) -> Result<SocketAddr> {
		let host = *self.config.server().host();

		let port = *self.config.server().port();

		Ok(SocketAddr::new(host, port))
	}

	pub async fn execute(&mut self) -> Result<()> {
		tracing::trace!("Executing Execution Plan");

		let socket = self.bind_addr()?;

		tracing::trace!(?socket);

		let router = routes::app_router(&self.config)?;

		axum::Server::bind(&socket)
			.serve(router.into_make_service())
			.await?;

		Ok(())
	}
}

#[cfg(test)]
mod execution_plan {
	use super::ExecutionPlan;

	#[cfg(test)]
	mod validate_version {
		use super::ExecutionPlan;

		#[test]
		fn valid_version() {
			let version = Some("0.0.0".parse().expect("hard-coded input should parse"));
			let requirement = "~0.0.0"
				.parse()
				.expect("hard-coded requirement should parse");
			assert!(ExecutionPlan::validate_version(&version, requirement).is_ok());
		}

		#[test]
		fn version_that_does_not_meet_requirements() {
			let version = Some("4.0.0".parse().expect("hard-coded input should parse"));
			let requirement = "~0.0.0"
				.parse()
				.expect("hard-coded requirement should parse");
			assert!(ExecutionPlan::validate_version(&version, requirement).is_err());
		}

		#[test]
		fn no_version_does_not_meet_requirements() {
			let version = None;
			let requirement = "~0.0.0"
				.parse()
				.expect("hard-coded requirement should parse");
			assert!(ExecutionPlan::validate_version(&version, requirement).is_err());
		}
	}
}
