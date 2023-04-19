use {
	crate::cli::{args::SqlAction, insert::insert_rows_from_json, select::select_rows},
	clap::Parser,
	color_eyre::{
		eyre::{bail as yeet, Context},
		Result,
	},
	schnosedb::models::*,
	sqlx::mysql::MySqlPoolOptions,
	tracing_subscriber::fmt::format::FmtSpan,
};

mod cli;

#[tokio::main]
async fn main() -> Result<()> {
	color_eyre::install()?;
	let args = cli::args::Args::parse();

	tracing_subscriber::fmt()
		.compact()
		.with_line_number(true)
		.with_file(true)
		.with_max_level(args.log_level)
		.with_span_events(FmtSpan::NEW)
		.init();

	let config = cli::config::get_config(&args.config_path).context("Failed to get config.")?;

	let database_connection = MySqlPoolOptions::new()
		.connect(&config.connection_string)
		.await
		.context("Failed to connect to database.")?;

	match args.sql_action {
		SqlAction::Select { table, limit } => {
			let json = match table.as_str() {
				"modes" => select_rows::<ModeRow>(&table, limit, &database_connection).await,
				"players" => select_rows::<PlayerRow>(&table, limit, &database_connection).await,
				"maps" => select_rows::<MapRow>(&table, limit, &database_connection).await,
				"mappers" => select_rows::<MapperRow>(&table, limit, &database_connection).await,
				"courses" => select_rows::<CourseRow>(&table, limit, &database_connection).await,
				"filters" => select_rows::<FilterRow>(&table, limit, &database_connection).await,
				"servers" => select_rows::<ServerRow>(&table, limit, &database_connection).await,
				"records" => select_rows::<RecordRow>(&table, limit, &database_connection).await,
				invalid_table => yeet!("`{invalid_table}` is not a valid table."),
			}?;

			use std::io::Write;
			std::io::stdout()
				.write_all(&json)
				.context("Failed to write data to STDOUT.")?;
		}
		SqlAction::Insert { table, json_path } if json_path.is_file() => {
			let json = std::fs::read_to_string(json_path).context("Failed to read JSON.")?;
			insert_rows_from_json(json, &table, &database_connection).await?;
		}
		SqlAction::Insert { table, json_path } if json_path.is_dir() => {
			for entry in std::fs::read_dir(json_path)? {
				let entry = entry?;
				let path = entry.path();
				if path.is_file() {
					let json = std::fs::read_to_string(path).context("Failed to read JSON.")?;
					insert_rows_from_json(json, &table, &database_connection).await?;
				}
			}
		}
		SqlAction::Insert { json_path, .. } => {
			yeet!("`{}` is neither a file nor a directory.", json_path.display())
		}
	};

	Ok(())
}
