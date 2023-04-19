use {
	clap::Parser,
	color_eyre::{eyre::Context, Result},
	config::get_config,
	elastic_scraper::elastic,
	elasticsearch::{
		auth::Credentials,
		http::{
			transport::{SingleNodeConnectionPool, TransportBuilder},
			Url,
		},
		Elasticsearch, SearchParts,
	},
	std::{path::PathBuf, time::Instant},
	tracing::{debug, info, trace, Level},
};

mod config;
mod save;

/// CLI Application to fetch data from zer0.k's KZ ElasticDB instance.
#[derive(Debug, Parser)]
struct Args {
	/// `RUST_LOG` level
	#[arg(long = "logs")]
	#[clap(default_value = "INFO")]
	log_level: Level,

	/// Path to the `config.toml` with Elastic connection secrets. See `config.example.toml`.
	#[arg(short, long = "config")]
	config_path: PathBuf,

	/// The output directory to store the Elastic data as JSON files.
	#[arg(short, long = "output")]
	output_directory: PathBuf,

	/// How many records to fetch per request.
	#[arg(long = "chunks")]
	#[clap(default_value = "3500")]
	chunk_size: usize,

	/// How many records to fetch before stopping.
	#[arg(long = "limit")]
	#[clap(default_value = "18446744073709551615")]
	fetch_limit: usize,
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

	let config = get_config(&args.config_path).context("Failed to get config.")?;
	trace!("Parsed config file.");

	let elastic_url =
		Url::parse(&config.connection_url).context("Failed to parse Elastic connection URL.")?;
	let elastic_credentials = Credentials::Basic(config.username, config.password);
	let elastic_connection_pool = SingleNodeConnectionPool::new(elastic_url);
	let elastic_transport = TransportBuilder::new(elastic_connection_pool)
		.auth(elastic_credentials)
		.build()
		.context("Failed to authorize with Elastic instance.")?;
	let elastic_client = Elasticsearch::new(elastic_transport.clone());
	trace!("Established connection to Elastic instance.");

	let start = Instant::now();

	let (scroll_id, (initial_records, initial_malformed_records)) = elastic::fetch_initial(
		args.chunk_size,
		SearchParts::Index(&["kzrecords2"]),
		&elastic_client,
	)
	.await
	.context("Failed to fetch initial data batch.")?;

	debug!("Initial ScrollID: {scroll_id}");

	let mut total_records = initial_records.len();
	let mut total_malformed_records = 0;
	let output_path = args.output_directory;
	let initial_output_path = output_path.clone().join("batch_0.json");
	let initial_malformed_output_path = output_path
		.clone()
		.join("malformed_batch_0.json");

	tokio::spawn(async move {
		save::save_as_json(initial_records, initial_output_path).await;
	});

	tokio::spawn(async move {
		save::save_as_json(initial_malformed_records, initial_malformed_output_path).await;
	});

	let limit = args.fetch_limit / args.chunk_size;
	let max_iterations =
		if args.fetch_limit == usize::MAX { String::from("∞") } else { limit.to_string() };
	let mut scroll_id = scroll_id;
	let mut count = 1;

	while let Ok((new_scroll_id, (records, malformed_records))) =
		elastic::scroll(&elastic_transport, &scroll_id).await
	{
		if records.is_empty() && malformed_records.is_empty() {
			break;
		}

		total_records += records.len();
		total_malformed_records += malformed_records.len();
		let malformed_output_path = output_path
			.clone()
			.join(format!("malformed_batch_{count}.json"));
		let output_path = output_path
			.clone()
			.join(format!("batch_{count}.json"));

		if !records.is_empty() {
			tokio::spawn(async move {
				save::save_as_json(records, output_path).await;
			});
		}

		if !malformed_records.is_empty() {
			tokio::spawn(async move {
				save::save_as_json(malformed_records, malformed_output_path).await;
			});
		}

		scroll_id = new_scroll_id;
		info!("\n[{count} / {max_iterations}] Iterations\n  • {total_records} total records\n  • {total_malformed_records} total malformed records");
		count += 1;
	}

	info!(
		"Done. Fetched {} records in total. (took {})",
		total_records + total_malformed_records,
		format_time(start.elapsed().as_secs_f64())
	);

	Ok(())
}

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
