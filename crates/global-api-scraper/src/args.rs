use {
	clap::{Parser, Subcommand},
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

	/// The data to fetch
	#[clap(subcommand)]
	pub data: Data,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Data {
	Players {
		#[arg(long = "start")]
		start_id: u32,

		/// Decrement the `start_id` each chunk
		#[arg(long)]
		backwards: bool,

		#[arg(long = "chunks")]
		chunk_size: usize,
	},

	Maps,
	Servers,

	Records {
		#[arg(long = "start")]
		start_id: u32,
	},

	Mappers,
	Filters,
}
