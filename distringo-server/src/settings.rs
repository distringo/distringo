use core::{fmt, str::FromStr};

use std::{collections::HashMap, net::IpAddr};

use serde::de;

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

fn de_version<'de, D: de::Deserializer<'de>>(
	deserializer: D,
) -> Result<Option<semver::Version>, D::Error> {
	struct StringVisitor;

	impl<'de> de::Visitor<'de> for StringVisitor {
		type Value = Option<semver::Version>;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			formatter.write_str("a string that can be parsed as a semver::Version using FromStr")
		}

		fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
		where
			E: de::Error,
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
	pub fn version(&self) -> &Option<semver::Version> {
		&self.version
	}

	pub fn server(&self) -> &ServerConfig {
		&self.server
	}

	fn datasets(&self) -> &Option<DatasetsConfig> {
		&self.datasets
	}

	pub fn shapefiles(&self) -> &ShapefilesConfig {
		&self.shapefiles
	}

	fn sessions(&self) -> &Option<SessionsConfig> {
		&self.sessions
	}
}

#[derive(serde::Deserialize)]
pub struct ServerConfig {
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
	pub fn host(&self) -> &IpAddr {
		&self.host
	}

	pub fn port(&self) -> &u16 {
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
