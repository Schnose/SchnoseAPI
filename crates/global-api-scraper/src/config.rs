use {
	crate::args::Args,
	color_eyre::{
		eyre::{bail as yeet, Context},
		Result,
	},
	serde::Deserialize,
};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
	pub database_url: String,
}

impl Config {
	#[tracing::instrument(level = "DEBUG", err(Debug))]
	pub fn load(args: &Args) -> Result<Self> {
		let config_file =
			std::fs::read_to_string(&args.config_path).context("Failed to read config file.")?;

		let config: Self = toml::from_str(&config_file).context("Failed to parse config file.")?;

		if config.database_url.is_empty() {
			yeet!("Database URL cannot be empty!");
		}

		Ok(config)
	}
}
