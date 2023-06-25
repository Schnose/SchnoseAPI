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
	pub elastic_url: String,
	pub username: String,
	pub password: String,
}

impl Config {
	#[tracing::instrument(level = "DEBUG", err(Debug))]
	pub async fn load(args: &Args) -> Result<Self> {
		let config_file = tokio::fs::read_to_string(&args.config_path)
			.await
			.context("Failed to read config file.")?;

		let config: Self = toml::from_str(&config_file).context("Failed to parse config file.")?;

		if config.elastic_url.is_empty() {
			yeet!("Elastic URL cannot be empty!");
		}

		if config.username.is_empty() {
			yeet!("username cannot be empty!");
		}

		if config.password.is_empty() {
			yeet!("password cannot be empty!");
		}

		Ok(config)
	}
}
