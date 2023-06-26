mod args;
mod config;

use {
	args::Args,
	clap::Parser,
	color_eyre::{eyre::Context, Result},
	config::Config,
	player_migrations::{select_players, Player},
	sqlx::{mysql::MySqlPoolOptions, postgres::PgPoolOptions, QueryBuilder},
	std::time::Instant,
	tracing::{info, trace},
	tracing_subscriber::util::SubscriberInitExt,
};

#[tokio::main]
async fn main() -> Result<()> {
	// Error handling
	color_eyre::install()?;

	// CLI arguments
	let mut args = Args::parse();

	// Logging
	just_trace::registry!().init();

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

	while let Ok(players) = select_players(args.offset, args.limit, &old_db).await {
		let mut query = QueryBuilder::new("INSERT INTO players (id, name, is_banned)");
		let amount = players.len();
		trace!(%amount, "Fetched players");

		query.push_values(
			players,
			|mut query,
			 Player {
			     id,
			     name,
			     is_banned,
			 }| {
				query.push_bind(id as i32).push_bind(name).push_bind(is_banned);
			},
		);

		query.build().execute(&new_db).await.context("Failed to insert players.")?;
		trace!(%amount, "Inserted players");

		total += amount;
		args.offset += args.limit as isize;
	}

	info!(took = ?start.elapsed(), %total, "Done.");

	Ok(())
}
