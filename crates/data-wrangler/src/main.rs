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
	tracing::{debug, info, trace, Level},
};

mod config;

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
pub struct Args {
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

	if !args.output_path.is_file() && !args.output_path.is_dir() {
		yeet!("`output` must be a file or a directory.");
	}

	let config = config::get_config(&args.config_path)?;

	let start = Instant::now();
	let total_objects = 0;

	let values: Vec<serde_json::Value> = if args.input_path.is_file() {
		debug!("Reading single file.");
		let json =
			std::fs::read_to_string(&args.input_path).context("Failed to read JSON file.")?;

		serde_json::from_str::<serde_json::Value>(&json)
			.context("Failed to parse JSON.")?
			.as_array()
			.unwrap()
			.to_owned()
	} else if args.input_path.is_dir() {
		debug!("Reading directory.");
		let mut values = Vec::new();
		for entry in std::fs::read_dir(&args.input_path)? {
			let entry = entry?;
			let path = entry.path();
			if path.is_file() {
				debug!("Reading file.");
				let json = std::fs::read_to_string(&path).context("Failed to read JSON file.")?;

				values.extend(
					serde_json::from_str::<serde_json::Value>(&json)
						.context("Failed to parse JSON.")?
						.as_array()
						.unwrap()
						.to_owned(),
				)
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

	let first_value = &values[0];
	dbg!(first_value);

	if serde_json::from_value::<ElasticRecord>(first_value.clone()).is_ok() {
		let mut records = Vec::new();

		for record in values {
			records.push(
				serde_json::from_value(record)
					.context("Failed to parse array of elastic records.")?,
			);
		}

		return parse_elastic_records::parse(records, &database_connection, &args)
			.await
			.context(format!("failed after {:?}", start.elapsed()));
	}

	if serde_json::from_value::<global_api::Record>(first_value.clone()).is_ok() {
		let mut records = Vec::new();

		for record in values {
			records
				.push(serde_json::from_value(record).context("Failed to parse elastic record.")?);
		}

		return parse_globalapi_records::parse(records, &args)
			.context(format!("failed after {:?}", start.elapsed()));
	}

	if serde_json::from_value::<global_api::Player>(first_value.clone()).is_ok() {
		let mut players = Vec::new();

		for player in values {
			players.push(serde_json::from_value(player).context("Failed to parse player.")?);
		}

		return parse_globalapi_players::parse(players, &args)
			.context(format!("failed after {:?}", start.elapsed()));
	}

	if serde_json::from_value::<parse_globalapi_servers::Server>(first_value.clone()).is_ok() {
		let mut servers = Vec::new();

		for server in values {
			servers.push(serde_json::from_value(server).context("Failed to parse server.")?);
		}

		return parse_globalapi_servers::parse(servers, &args)
			.context(format!("failed after {:?}", start.elapsed()));
	}

	info!("Done. Parsed {} objectcs in total. (took {:?})", total_objects, start.elapsed());

	Ok(())
}
