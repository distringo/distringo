use core::{fmt, str::FromStr};

use std::collections::HashMap;

use serde::de;

mod server;
pub use server::*;

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
	datasets: DatasetsConfig,
	shapefiles: ShapefilesConfig,
	sessions: SessionsConfig,
}

impl AppConfig {
	pub fn version(&self) -> &Option<semver::Version> {
		&self.version
	}

	pub fn server(&self) -> &ServerConfig {
		&self.server
	}

	pub fn datasets(&self) -> &DatasetsConfig {
		&self.datasets
	}

	pub fn shapefiles(&self) -> &ShapefilesConfig {
		&self.shapefiles
	}

	pub fn sessions(&self) -> &SessionsConfig {
		&self.sessions
	}
}

#[derive(serde::Deserialize, Default)]
pub struct DatasetsConfig(HashMap<String, DatasetConfig>);

#[derive(serde::Deserialize, Default)]
pub struct DatasetConfig {}

// TODO: Move into shapefiles route section
#[derive(Debug, serde::Deserialize, Default)]
pub struct ShapefilesConfig(HashMap<String, ShapefileConfig>);

impl core::convert::AsRef<HashMap<String, ShapefileConfig>> for ShapefilesConfig {
	fn as_ref(&self) -> &HashMap<String, ShapefileConfig> {
		&self.0
	}
}

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
pub struct SessionsConfig(HashMap<String, SessionConfig>);

#[derive(serde::Deserialize, Default)]
pub struct SessionConfig {}
