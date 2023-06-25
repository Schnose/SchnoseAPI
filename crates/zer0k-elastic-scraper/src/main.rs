mod args;
mod config;

use {
	args::Args,
	clap::Parser,
	color_eyre::{eyre::Context, Result},
	config::Config,
	elasticsearch::{
		auth::Credentials,
		http::{
			transport::{SingleNodeConnectionPool, TransportBuilder},
			Url,
		},
		Elasticsearch, SearchParts,
	},
	std::time::Instant,
	tracing::{debug, error, info},
	tracing_subscriber::util::SubscriberInitExt,
	zer0k_elastic_scraper::{elastic, elastic::Payload, utils::save_json},
};

#[tokio::main]
async fn main() -> Result<()> {
	// Error handling
	color_eyre::install()?;

	// CLI arguments
	let args = Args::parse();

	// Logging
	just_trace::registry!().init();

	let config = Config::load(&args).await?;

	let url = Url::parse(&config.elastic_url).context("`elastic_url` is not an actual URL.")?;

	let credentials = Credentials::Basic(config.username, config.password);
	let pool = SingleNodeConnectionPool::new(url);
	let transport = TransportBuilder::new(pool)
		.auth(credentials)
		.build()
		.context("Failed to construct elastic transport.")?;

	let client = Elasticsearch::new(transport.clone());

	let start = Instant::now();

	let Payload {
		scroll_id,
		records,
		failures,
	} = elastic::fetch_initial(args.chunk_size, SearchParts::Index(&["kzrecords2"]), &client)
		.await
		.context("Failed to fetch initial payload.")?;

	let records_output_path = args.output_path.join("batch_0.json");
	let failures_output_path = args.output_path.join("failures_0.json");

	debug!(%scroll_id, "Received initial response.");

	let mut total_records = records.len();
	let mut total_failures = failures.len();

	tokio::spawn(async move {
		if let Err(err) = save_json(&records, &records_output_path).await {
			error!(?err, "Failed to write json to disk");
			panic!();
		}
	});

	tokio::spawn(async move {
		if let Err(err) = save_json(&failures, &failures_output_path).await {
			error!(?err, "Failed to write json to disk");
			panic!();
		}
	});

	let max = match args.fetch_limit {
		usize::MAX => String::from("∞"),
		limit => (limit / args.chunk_size).to_string(),
	};

	let mut scroll_id = scroll_id;
	let mut count = 1;

	while let Ok(Payload {
		scroll_id: new_scroll_id,
		records,
		failures,
	}) = elastic::fetch(&scroll_id, &transport).await
	{
		if records.is_empty() && failures.is_empty() {
			info!("No more records! We're done.");
			break;
		}

		total_records += records.len();
		total_failures += failures.len();

		let records_output_path = args.output_path.join(format!("batch_{count}.json"));
		let failures_output_path = args.output_path.join(format!("failures_{count}.json"));

		if !records.is_empty() {
			tokio::spawn(async move {
				if let Err(err) = save_json(&records, &records_output_path).await {
					error!(?err, "Failed to write json to disk");
					panic!();
				}
			});
		}

		if !failures.is_empty() {
			tokio::spawn(async move {
				if let Err(err) = save_json(&failures, &failures_output_path).await {
					error!(?err, "Failed to write json to disk");
					panic!();
				}
			});
		}

		scroll_id = new_scroll_id;

		info!(total = %total_records, failures = %total_failures, "{count} / {max}");

		count += 1;
	}

	info!(
		took = ?start.elapsed(),
		"Done. Fetched {total} records in total ({failures} failures).",
		total = total_records + total_failures,
		failures = total_failures,
	);

	Ok(())
}
