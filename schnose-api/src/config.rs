use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
	pub connection_string: String,
	pub addr: [u8; 4],
	pub port: u16,
}
