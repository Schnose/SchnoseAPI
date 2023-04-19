use {
	clap::{Parser, Subcommand},
	std::path::PathBuf,
	tracing::Level,
};

#[derive(Debug, Parser)]
pub struct Args {
	/// `RUST_LOG` level
	#[arg(long = "logs")]
	#[clap(default_value = "INFO")]
	pub log_level: Level,

	/// Path to the `config.toml` with a MySQL connection string.
	#[arg(short, long = "config")]
	pub config_path: PathBuf,

	/// What action to perform on the database.
	#[clap(subcommand)]
	pub sql_action: SqlAction,
}

#[derive(Debug, Clone, Subcommand)]
pub enum SqlAction {
	/// Will select `limit` rows from `table` and write the results as JSON to STDOUT.
	Select {
		#[arg(long)]
		table: String,
		#[arg(long)]
		limit: Option<usize>,
	},

	/// Will read rows as JSON from `json_path` and insert them into `table`.
	/// If `json_path` is a directory, it will read all files in that directory individually.
	Insert {
		#[arg(long)]
		table: String,
		#[arg(long = "json")]
		json_path: PathBuf,
	},
}
