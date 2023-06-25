use {clap::Parser, std::path::PathBuf};

/// A crate to fetch information from an ElasticSearch instance.
#[derive(Debug, Clone, Parser)]
pub struct Args {
	/// The config file containing connection details.
	///
	/// See the example file.
	#[arg(short, long = "config")]
	#[clap(default_value = "./config.toml")]
	pub config_path: PathBuf,

	/// The directory in which the output json files should be stored.
	#[arg(short, long = "output")]
	#[clap(default_value = "./elastic-output/")]
	pub output_path: PathBuf,

	/// How many items to fetch per request.
	#[arg(long)]
	#[clap(default_value_t = 3500)]
	pub chunk_size: usize,

	/// How many records to fetch before stopping automatically.
	#[arg(short = 'l', long = "limit")]
	#[clap(default_value_t = usize::MAX)]
	pub fetch_limit: usize,
}
