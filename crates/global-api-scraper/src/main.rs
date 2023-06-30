use global_api_scraper::{fetch_filters, fetch_mappers, fetch_records, fetch_servers};

mod args;
mod config;

use {
	crate::{args::Args, config::Config},
	args::Data,
	clap::Parser,
	color_eyre::{eyre::Context, Result},
	global_api_scraper::{fetch_maps, fetch_players},
	sqlx::postgres::PgPoolOptions,
	std::time::Instant,
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

	let gokz_client = gokz_rs::Client::new();

	let pool = PgPoolOptions::new()
		.connect(&config.database_url)
		.await
		.context("Failed to establish database connection.")?;

	let total = match args.data {
		Data::Players {
			start_offset,
			backwards,
			chunk_size,
		} => fetch_players(start_offset, backwards, chunk_size, &gokz_client, &pool)
			.await
			.context("Failed to fetch players."),

		Data::Maps => {
			fetch_maps(&gokz_client, &pool).await.context("Failed to fetch maps and courses.")
		}

		Data::Servers => {
			fetch_servers(&gokz_client, &pool).await.context("Failed to fetch servers.")
		}

		Data::Records {
			start_id,
		} => {
			fetch_records(start_id, &gokz_client, &pool)
				.await
				.context("Failed to fetch records.")?;

			unreachable!("Record fetching is an infinite loop.");
		}

		Data::Mappers => {
			fetch_mappers(&gokz_client, &pool).await.context("Failed to fetch mappers.")
		}

		Data::Filters => {
			fetch_filters(&gokz_client, &pool).await.context("Failed to fetch filters.")
		}
	}?;

	info!(took = ?start.elapsed(), %total, "Done.");

	Ok(())
}
