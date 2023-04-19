use {
	color_eyre::{
		eyre::{bail as yeet, Context},
		Result,
	},
	serde::{Deserialize, Serialize},
	std::path::Path,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
	pub connection_url: String,
	pub username: String,
	pub password: String,
}

pub fn get_config(config_path: &Path) -> Result<Config> {
	let config_file =
		std::fs::read_to_string(config_path).context("Failed to read config file.")?;

	let config: Config = toml::from_str(&config_file).context("Failed to parse config file.")?;

	if config.connection_url.is_empty() {
		yeet!("`connection_url` must not be empty!");
	}

	if config.username.is_empty() {
		yeet!("`username` must not be empty!");
	}

	if config.password.is_empty() {
		yeet!("`password` must not be empty!");
	}

	Ok(config)
}
