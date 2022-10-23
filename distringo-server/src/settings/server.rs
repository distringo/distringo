use std::net::{IpAddr, Ipv6Addr};

#[derive(serde::Deserialize)]
pub struct ServerConfig {
	host: IpAddr,
	port: u16,
}

impl Default for ServerConfig {
	fn default() -> Self {
		Self {
			host: IpAddr::V6(Ipv6Addr::from([0; 16])),
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

#[cfg(test)]
mod tests {
	use super::ServerConfig;

	#[cfg(test)]
	mod default {
		use super::ServerConfig;

		#[test]
		fn default_is_localhost_2020() {
			let default = ServerConfig::default();
			assert_eq!(
				default.host,
				"::"
					.parse::<std::net::Ipv6Addr>()
					.expect("hard-coded strings should parse")
			);
			assert_eq!(default.port, 2020);
		}
	}

	#[test]
	fn host_works() {
		let config: ServerConfig = ServerConfig {
			host: std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
			port: 0_u16,
		};

		assert_eq!(
			config.host(),
			&std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))
		);
	}
}
