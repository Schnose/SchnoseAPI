use {clap::Parser, std::path::PathBuf};

#[derive(Debug, Clone, Parser)]
pub struct Args {
	/// The config file containing connection details.
	///
	/// See the example file.
	#[arg(short, long = "config")]
	#[clap(default_value = "./config.toml")]
	pub config_path: PathBuf,

	/// The amount of players to transfer at a time
	#[arg(long)]
	#[clap(default_value_t = 2000)]
	pub limit: usize,

	/// The start offset
	#[arg(long)]
	#[clap(default_value_t = 0)]
	pub offset: isize,
}
