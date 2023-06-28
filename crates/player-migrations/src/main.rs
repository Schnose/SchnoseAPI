mod args;
mod config;

use {
	args::Args,
	clap::Parser,
	color_eyre::{eyre::Context, Result},
	config::Config,
	player_migrations::{insert_players, select_players},
	sqlx::{mysql::MySqlPoolOptions, postgres::PgPoolOptions},
	std::time::Instant,
	tracing::info,
	tracing_subscriber::util::SubscriberInitExt,
};

#[tokio::main]
async fn main() -> Result<()> {
	// Error handling
	color_eyre::install()?;

	// CLI arguments
	let mut args = Args::parse();

	// Logging
	just_trace::registry!(minimal).init();

	let config = Config::load(&args)?;

	let start = Instant::now();

	let old_db = MySqlPoolOptions::new()
		.connect(&config.old_database_url)
		.await
		.context("Failed to connect to old database.")?;

	info!("Connected to old database.");

	let new_db = PgPoolOptions::new()
		.connect(&config.new_database_url)
		.await
		.context("Failed to connect to new database.")?;

	info!("Connected to new database.");

	let mut total = 0;

	while let Ok(Some(players)) = select_players(args.offset, args.limit, &old_db).await {
		total += insert_players(players, &new_db).await?;
		args.offset += args.limit as isize;
	}

	info!(took = ?start.elapsed(), %total, "Done.");

	Ok(())
}
