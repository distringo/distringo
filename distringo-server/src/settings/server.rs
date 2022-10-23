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
