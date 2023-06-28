use {clap::Parser, std::path::PathBuf};

#[derive(Debug, Clone, Parser)]
pub struct Args {
	/// The config file containing connection details.
	///
	/// See the example file.
	#[arg(short, long = "config")]
	#[clap(default_value = "./config.toml")]
	pub config_path: PathBuf,

	/// Whether to run on 0.0.0.0
	#[arg(long)]
	pub public: bool,

	/// The port to run the API on
	#[arg(short, long)]
	pub port: Option<u16>,

	/// Print logs
	#[arg(long)]
	pub debug: bool,
}
