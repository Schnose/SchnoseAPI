mod args;
mod config;

use {
	args::{Args, Input, InputKind},
	clap::Parser,
	color_eyre::{eyre::Context, Result},
	config::Config,
	data_wrangler::{
		insert_elastic_records, insert_maps, insert_players, insert_records, insert_servers,
	},
	sqlx::postgres::PgPoolOptions,
	std::{path::Path, time::Instant},
	tracing::info,
	tracing_subscriber::util::SubscriberInitExt,
};

#[tokio::main]
async fn main() -> Result<()> {
	// Error handling
	color_eyre::install()?;

	// CLI arguments
	let args = Args::parse();

	// Logging
	just_trace::registry!().init();

	let config = Config::load(&args)?;

	let start = Instant::now();

	match args.input {
		Input::Json {
			path,
			kind,
		} => {
			process_json(&path, kind, &config).await.context("Failed to process json.")?;
		}
	};

	info!(took = ?start.elapsed(), "Done.");

	Ok(())
}

#[tracing::instrument(level = "INFO", skip(config), err(Debug))]
async fn process_json(path: &Path, kind: InputKind, config: &Config) -> Result<()> {
	let input = tokio::fs::read_to_string(path).await.context("Failed to read input.")?;
	let pool = PgPoolOptions::new()
		.connect(&config.database_url)
		.await
		.context("Failed to establish database connection.")?;

	let processed = match kind {
		InputKind::Players => {
			let players = serde_json::from_str(&input).context("Failed to parse players.")?;
			insert_players(players, &pool)
				.await
				.context("Failed to insert players into the database.")
		}

		InputKind::Maps => {
			let maps = serde_json::from_str(&input).context("Failed to parse maps.")?;
			insert_maps(maps, &pool).await.context("Failed to insert maps into the database.")
		}

		InputKind::Servers => {
			let servers = serde_json::from_str(&input).context("Failed to parse servers.")?;
			insert_servers(servers, &pool)
				.await
				.context("Failed to insert servers into the database.")
		}

		InputKind::Records => {
			let records = serde_json::from_str(&input).context("Failed to parse records.")?;
			insert_records(records, &pool)
				.await
				.context("Failed to insert records into the database.")
		}

		InputKind::ElasticRecords => {
			let records = serde_json::from_str(&input).context("Failed to parse records.")?;
			insert_elastic_records(records, &pool)
				.await
				.context("Failed to insert records into the database.")
		}
	}?;

	info!("done processing {processed} items");

	Ok(())
}
