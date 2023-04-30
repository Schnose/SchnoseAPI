use {
	clap::{Parser, Subcommand},
	color_eyre::{eyre::Context, Result},
	global_api_scraper::{fetch_players, fetch_records, RecordID},
	sqlx::mysql::MySqlPoolOptions,
	std::{path::PathBuf, time::Instant},
	tracing::{info, trace, Level},
};

mod config;

/// CLI Application to fetch records from the GlobalAPI.
#[derive(Debug, Parser)]
struct Args {
	/// `RUST_LOG` level
	#[arg(long = "logs")]
	#[clap(default_value = "INFO")]
	log_level: Level,

	/// Path to the `config.toml` with MySQL connection string.
	#[arg(short, long = "config")]
	config_path: PathBuf,

	/// What kind of data to scrape.
	#[clap(subcommand)]
	data: Data,
}

#[derive(Debug, Subcommand)]
enum Data {
	Records {
		/// The record id to start scraping from. If no id is given, the highest id currently in the
		/// database will be used.
		#[arg(short, long = "start")]
		start_id: Option<u32>,
	},

	Players {
		/// The offset to start scraping from. This will **decrease** with every request, so it should
		/// start out high.
		#[arg(short, long = "start")]
		start_offset: u32,
	},
}

#[tokio::main]
async fn main() -> Result<()> {
	color_eyre::install()?;
	let args = Args::parse();
	trace!("Parsed CLI args.");

	tracing_subscriber::fmt()
		.compact()
		.with_line_number(false)
		.with_file(false)
		.without_time()
		.with_max_level(args.log_level)
		.init();

	info!("Initialized logging with level `{}`.", args.log_level);

	let config = config::get_config(&args.config_path)?;

	let start = Instant::now();

	let database_connection = MySqlPoolOptions::new()
		.connect(&config.connection_string)
		.await
		.context("Failed to establish database connection.")?;

	let gokz_client = gokz_rs::Client::new();

	match args.data {
		Data::Records { start_id } => {
			let start_id = match start_id {
				Some(id) => id,
				None => {
					sqlx::query_as::<_, RecordID>("SELECT MAX(id) AS id FROM records")
						.fetch_one(&database_connection)
						.await
						.context("Failed to get initial record id from database.")?
						.0 + 1
				}
			};

			fetch_records::fetch_and_insert(start_id, &gokz_client, &database_connection).await?;
			info!("Done. (took {:?})", start.elapsed());
		}

		Data::Players { start_offset } => {
			fetch_players::fetch_and_insert(start_offset, &gokz_client, &database_connection)
				.await?;
			info!("Done. (took {:?})", start.elapsed());
		}
	};

	Ok(())
}
