use {
	color_eyre::{
		eyre::{bail as yeet, Context},
		Result,
	},
	serde::{Deserialize, Serialize},
	std::path::Path,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
	pub connection_string: String,
}

pub fn get_config(config_path: &Path) -> Result<Config> {
	let config_file =
		std::fs::read_to_string(config_path).context("Failed to read config file.")?;

	let config: Config = toml::from_str(&config_file).context("Failed to parse config file.")?;

	if config.connection_string.is_empty() {
		yeet!("`connection_string` must not be empty!");
	}

	Ok(config)
}
