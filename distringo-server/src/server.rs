use std::{
	collections::HashMap,
	net::{IpAddr, SocketAddr},
};

use core::{fmt, str::FromStr};

use crate::Result;

// version: "0.0.0"
//
// server:
//   host: "::"
//   port: 2020
//   data: ./data/
//
// datasets:
//   in2010-pl94_171:
//     schema: schemas/2010/pl94_171.yaml
//     files:
//       geo: ingeo2010.pl
//       file01: in000012010.pl
//       file02: in000022010.pl
//       file03: in000032010.pl
//
// shapefiles:
//   tl_2010_18157_tabblock:
//     type: tabular_block
//     file: data/tl_2010_18157_tabblock10.min.geojson
//
// sessions:
//   tippecanoe-2010:
//     name: "tippecanoe county 2010"
//     datasets:
//       - in2010-pl94_171
//     shapefiles:
//       - tl_2010_18157_tabblock

#[derive(serde::Deserialize)]
struct AppConfig {
	server: ServerConfig,
	datasets: DatasetsConfig,
	shapefiles: ShapefilesConfig,
	sessions: SessionsConfig,
}

#[derive(serde::Deserialize)]
struct ServerConfig {}

#[derive(serde::Deserialize)]
struct DatasetsConfig {}

#[derive(serde::Deserialize)]
struct ShapefilesConfig {}

#[derive(serde::Deserialize)]
struct SessionsConfig {}

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

type DatasetId = String;

#[derive(Default)]
pub struct ExecutionPlan {
	config: config::Config,
	datasets: HashMap<DatasetId, ()>,
}

impl From<config::Config> for ExecutionPlan {
	fn from(config: config::Config) -> Self {
		Self {
			config,
			..Default::default()
		}
	}
}

impl ExecutionPlan {
	fn validate_version(version: &str) -> Result<(), AppConfigError> {
		const VERSION_REQUIREMENT: &str = "~0.0.0";

		let requirement = semver::VersionReq::parse(VERSION_REQUIREMENT)
			.expect("internally-generated version requirement was invalid");

		let version: semver::Version = version.parse()?;

		if requirement.matches(&version) {
			Ok(())
		} else {
			Err(AppConfigError::InvalidVersion)
		}
	}

	pub async fn validate(&self) -> Result<(), AppConfigError> {
		let config = &self.config;

		// Verify that the version is valid.
		let config_version = config.get_string("version")?;
		Self::validate_version(&config_version)?;

		// Load up all the dataset configurations.
		for (identifier, value) in config.get_table("datasets")? {
			println!("{:?}, {:?}", identifier, value);
		}

		// Load up all the shapefile cofigurations.
		for (identifier, value) in config.get_table("shapefiles")? {
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

	fn bind_addr(&self) -> Result<SocketAddr> {
		let host: IpAddr = self
			.config
			.get_string("server.host")?
			.parse()
			.map_err(|_| crate::RuntimeError::InvalidServerHost)?;

		let port: u16 = self
			.config
			.get_int("server.port")?
			.try_into()
			.map_err(|_| crate::RuntimeError::InvalidServerPort)?;

		Ok(SocketAddr::new(host, port))
	}

	pub async fn execute(&mut self) -> Result<()> {
		tracing::trace!("Executing Execution Plan");

		let config: &config::Config = &self.config;

		let socket = self.bind_addr()?;

		tracing::trace!(?socket);

		let router = routes::app_router(config)?;

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
			let config_version = "0.0.0";
			assert!(ExecutionPlan::validate_version(config_version).is_ok());
		}

		#[test]
		fn non_version_err() {
			let config_version = "not a version";
			assert!(ExecutionPlan::validate_version(config_version).is_err());
		}
	}
}
