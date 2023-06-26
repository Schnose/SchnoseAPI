use {
	clap::{Parser, Subcommand, ValueEnum},
	std::path::PathBuf,
};

#[derive(Debug, Clone, Parser)]
pub struct Args {
	/// The config file containing connection details.
	///
	/// See the example file.
	#[arg(short, long = "config")]
	#[clap(default_value = "./config.toml")]
	pub config_path: PathBuf,

	/// The kind of input
	#[clap(subcommand)]
	pub input: Input,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Input {
	Json {
		#[arg(long)]
		path: PathBuf,

		#[arg(long)]
		kind: InputKind,
	},
}

#[derive(Debug, Clone, ValueEnum)]
pub enum InputKind {
	Players,
	Maps,
	Servers,
	Records,
	ElasticRecords,
}
