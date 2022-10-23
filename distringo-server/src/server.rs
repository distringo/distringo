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

fn de_version<'de, D: serde::de::Deserializer<'de>>(
	deserializer: D,
) -> Result<Option<semver::Version>, D::Error> {
	struct StringVisitor;

	impl<'de> serde::de::Visitor<'de> for StringVisitor {
		type Value = Option<semver::Version>;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			formatter.write_str("a string that can be parsed as a semver::Version using FromStr")
		}

		fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
		where
			E: serde::de::Error,
		{
			match semver::Version::from_str(v) {
				Ok(v) => Ok(Some(v)),
				Err(e) => Err(E::custom(e)),
			}
		}
	}

	deserializer.deserialize_string(StringVisitor)
}

#[derive(serde::Deserialize, Default)]
pub struct AppConfig {
	#[serde(deserialize_with = "de_version")]
	version: Option<semver::Version>,
	server: ServerConfig,
	datasets: Option<DatasetsConfig>,
	shapefiles: ShapefilesConfig,
	sessions: Option<SessionsConfig>,
}

impl AppConfig {
	fn version(&self) -> &Option<semver::Version> {
		&self.version
	}

	fn server(&self) -> &ServerConfig {
		&self.server
	}

	fn datasets(&self) -> &Option<DatasetsConfig> {
		&self.datasets
	}

	fn shapefiles(&self) -> &ShapefilesConfig {
		&self.shapefiles
	}

	fn sessions(&self) -> &Option<SessionsConfig> {
		&self.sessions
	}
}

#[derive(serde::Deserialize)]
struct ServerConfig {
	host: IpAddr,
	port: u16,
}

impl Default for ServerConfig {
	fn default() -> Self {
		Self {
			host: IpAddr::V6(std::net::Ipv6Addr::from([0; 16])),
			port: 2020_u16,
		}
	}
}

impl ServerConfig {
	fn host(&self) -> &IpAddr {
		&self.host
	}

	fn port(&self) -> &u16 {
		&self.port
	}
}

#[derive(serde::Deserialize, Default)]
struct DatasetsConfig {}

// TODO: Move into shapefiles route section
#[derive(Debug, serde::Deserialize, Default)]
pub struct ShapefilesConfig(HashMap<String, ShapefileConfig>);

#[derive(Debug, serde::Deserialize, Default)]
pub struct ShapefileConfig {
	ty: Option<ShapefileType>,
	file: String,
}

#[derive(Debug, serde::Deserialize)]
pub enum ShapefileType {
	TabularBlock,
}

#[derive(serde::Deserialize, Default)]
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
	config: AppConfig,
}

impl From<config::Config> for ExecutionPlan {
	fn from(config: config::Config) -> Self {
		todo!()
		// Self {
		// 	config,
		// 	..Default::default()
		// }
	}
}

impl From<AppConfig> for ExecutionPlan {
	fn from(config: AppConfig) -> Self {
		Self {
			config,
			..Default::default()
		}
	}
}

impl ExecutionPlan {
	fn validate_version(version: &Option<semver::Version>) -> Result<(), AppConfigError> {
		const VERSION_REQUIREMENT: &str = "~0.0.0";

		let requirement = semver::VersionReq::parse(VERSION_REQUIREMENT)
			.expect("internally-generated version requirement was invalid");

		if let Some(version) = version {
			if requirement.matches(&version) {
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

		// Verify that the version is valid.
		let config_version = self.config.version();
		Self::validate_version(config_version)?;

		// Load up all the dataset configurations.
		// for (identifier, value) in config.get_table("datasets")? {
		// 	println!("{:?}, {:?}", identifier, value);
		// }

		// Load up all the shapefile cofigurations.
		// for (identifier, value) in config.get_table("shapefiles")? {
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
			let config_version = Some("0.0.0".parse().expect("hard-coded input should parse"));
			assert!(ExecutionPlan::validate_version(&config_version).is_ok());
		}

		#[test]
		fn version_that_does_not_meet_requirements() {
			let config_version = Some("4.0.0".parse().expect("hard-coded input should parse"));
			assert!(ExecutionPlan::validate_version(&config_version).is_err());
		}

		#[test]
		fn no_version_does_not_meet_requirements() {
			let config_version = None;
			assert!(ExecutionPlan::validate_version(&config_version).is_err());
		}
	}
}
