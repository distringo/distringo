use std::net::{IpAddr, Ipv6Addr, SocketAddr};

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

	pub fn bind_addr(&self) -> SocketAddr {
		let host = self.host;
		let port = self.port;

		SocketAddr::new(host, port)
	}
}

#[cfg(test)]
mod tests {
	use std::net::{IpAddr, Ipv4Addr, SocketAddr};

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

	const fn basic_config_ipv4_addr() -> Ipv4Addr {
		std::net::Ipv4Addr::new(127, 0, 0, 1)
	}

	const fn basic_config_ip_addr() -> IpAddr {
		std::net::IpAddr::V4(basic_config_ipv4_addr())
	}

	const fn basic_config_port() -> u16 {
		32040_u16
	}

	const fn basic_config() -> ServerConfig {
		ServerConfig {
			host: basic_config_ip_addr(),
			port: basic_config_port(),
		}
	}

	#[test]
	fn host_works() {
		let config: ServerConfig = basic_config();

		assert_eq!(config.host(), &basic_config_ip_addr());
	}

	#[test]
	fn port_works() {
		let config: ServerConfig = basic_config();

		assert_eq!(config.port(), &basic_config_port());
	}

	#[test]
	fn bind_addr() {
		let config: ServerConfig = basic_config();

		assert_eq!(
			config.bind_addr(),
			SocketAddr::new(basic_config_ip_addr(), basic_config_port())
		);
	}
}
