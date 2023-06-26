use {
	super::Args,
	color_eyre::{eyre::Context, Result},
	serde::Deserialize,
	std::net::SocketAddr,
};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
	pub database_url: String,
	pub ip_address: SocketAddr,
}

impl Config {
	pub async fn load(args: &Args) -> Result<Self> {
		let config_file = tokio::fs::read_to_string(&args.config_path)
			.await
			.context("Failed to load config file.")?;

		let mut config: Self =
			toml::from_str(&config_file).context("Failed to parse config file.")?;

		if config.database_url.is_empty() {
			config.database_url = std::env::var("DATABASE_URL").context(
				"Missing `database_url` in config file or `DATABASE_URL` environment variable.",
			)?;
		}

		if let Some(port) = args.port {
			config.ip_address.set_port(port);
		}

		Ok(config)
	}
}
