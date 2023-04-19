use {
	clap::Parser,
	color_eyre::{
		eyre::{bail as yeet, Context},
		Result,
	},
	elastic_scraper::elastic::ElasticRecord,
	gokz_rs::global_api,
	sqlx::mysql::MySqlPoolOptions,
	std::{path::PathBuf, time::Instant},
	tracing::{info, trace, Level},
};

mod config;
mod save;

mod parse_elastic_records;

mod parse_globalapi_players;
mod parse_globalapi_records;
mod parse_globalapi_servers;

/// CLI Application to parse data from zer0.k's KZ ElasticDB instance or the GlobalAPI.
/// Currently supports:
///   - Elastic Records
///   - GlobalAPI Records
///   - GlobalAPI Players
///   - GlobalAPI Servers
#[derive(Debug, Parser)]
struct Args {
	/// `RUST_LOG` level
	#[arg(long = "logs")]
	#[clap(default_value = "INFO")]
	log_level: Level,

	/// Path to the `config.toml` with MySQL connection string.
	#[arg(short, long = "config")]
	config_path: PathBuf,

	/// Path to input file / directory containing JSON data to parse
	#[arg(short, long = "input")]
	input_path: PathBuf,

	/// Path to output file to store the JSON
	#[arg(short, long = "output")]
	output_path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
	color_eyre::install()?;
	let args = Args::parse();
	trace!("Parsed CLI args.");

	tracing_subscriber::fmt()
		.compact()
		.with_line_number(true)
		.with_file(true)
		.with_max_level(args.log_level)
		.init();

	info!("Initialized logging with level `{}`.", args.log_level);

	let config = config::get_config(&args.config_path)?;

	let start = Instant::now();
	let mut total_records = 0;

	let values: Vec<serde_json::Value> = if args.input_path.is_file() {
		let json =
			std::fs::read_to_string(&args.input_path).context("Failed to read JSON file.")?;

		vec![serde_json::from_str(&json).context("Failed to parse JSON.")?]
	} else if args.input_path.is_dir() {
		let mut values = Vec::new();
		for entry in std::fs::read_dir(&args.input_path)? {
			let entry = entry?;
			let path = entry.path();
			if path.is_file() {
				let json = std::fs::read_to_string(&path).context("Failed to read JSON file.")?;

				values.push(serde_json::from_str(&json).context("Failed to parse JSON.")?)
			}
		}
		values
	} else {
		yeet!("`input` is neither a file nor a directory.");
	};

	let database_connection = MySqlPoolOptions::new()
		.connect(&config.connection_string)
		.await
		.context("Failed to establish database connection.")?;

	let values = serde_json::Value::Array(values);
	let first_value = &values[0];

	if serde_json::from_value::<ElasticRecord>(first_value.clone()).is_ok() {
		return parse_elastic_records::parse(
			serde_json::from_value(values).context("Failed to parse array of elastic records.")?,
			&database_connection,
			&args,
		)
		.await;
	}

	if serde_json::from_value::<global_api::Record>(first_value.clone()).is_ok() {
		return parse_globalapi_records::parse(
			serde_json::from_value(values).context("Failed to parse array of elastic records.")?,
			&database_connection,
			&args,
		);
	}

	if serde_json::from_value::<global_api::Player>(first_value.clone()).is_ok() {
		return parse_globalapi_players::parse(
			serde_json::from_value(values).context("Failed to parse array of elastic records.")?,
			&database_connection,
			&args,
		);
	}

	if serde_json::from_value::<global_api::Server>(first_value.clone()).is_ok() {
		return parse_globalapi_servers::parse(
			serde_json::from_value(values).context("Failed to parse array of elastic records.")?,
			&database_connection,
			&args,
		);
	}

	info!(
		"Done. Parsed {} records in total. (took {})",
		total_records,
		format_time(start.elapsed().as_secs_f64())
	);

	Ok(())
}

#[derive(sqlx::FromRow)]
struct MapID(u16);

#[derive(sqlx::FromRow)]
struct ServerID(u16);

fn format_time(seconds: f64) -> String {
	let hours = (seconds / 3600.0) as u8;
	let minutes = ((seconds % 3600.0) / 60.0) as u8;
	let seconds = seconds % 60.0;

	let mut formatted = format!("{minutes:02}:{seconds:06.3}");

	if hours > 0 {
		formatted = format!("{hours:02}:{formatted}");
	}

	formatted
}
