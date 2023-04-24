//!

#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms)]
#![warn(clippy::style, clippy::cognitive_complexity, clippy::complexity)]
#![deny(clippy::perf, clippy::correctness)]

use {
	axum::{routing::get, Router, Server},
	clap::Parser,
	color_eyre::{eyre::Context, Result},
	config::Config,
	state::APIState,
	std::{net::SocketAddr, path::PathBuf},
	time::macros::format_description as time,
	tracing::{info, Level},
	tracing_subscriber::fmt::{format::FmtSpan, time::UtcTime},
};

mod config;
mod state;

#[derive(Debug, Parser)]
struct Args {
	/// Path to a `config.toml` file.
	#[arg(short, long = "config")]
	#[clap(default_value = "./config.toml")]
	config_path: PathBuf,

	/// `RUST_LOG` level
	#[arg(long = "logs")]
	#[clap(default_value = "INFO")]
	log_level: Level,
}

#[tokio::main]
async fn main() -> Result<()> {
	color_eyre::install()?;

	let Args { config_path, log_level } = Args::parse();

	tracing_subscriber::fmt()
		.compact()
		.with_file(true)
		.with_line_number(true)
		.with_level(true)
		.with_max_level(log_level)
		.with_timer(UtcTime::new(time!("[[[year]-[month]-[day] | [hour]:[minute]:[second]]")))
		.with_span_events(FmtSpan::NEW)
		.init();

	let config_file =
		std::fs::read_to_string(&config_path).context("Failed to read config file.")?;
	let Config { connection_string, addr, port } =
		toml::from_str(&config_file).context("Failed to parse config file.")?;

	let state = APIState::new(&connection_string)
		.await
		.context("Failed to initialize State.")?;

	let addr = SocketAddr::from((addr, port));
	let server = Server::bind(&addr);

	info!("Listening on {addr}.");

	let router = Router::new()
		.route("/", get(|| async { "(͡ ͡° ͜ つ ͡͡°)" }))
		.with_state(state);

	server
		.serve(router.into_make_service())
		.await
		.context("Failed to run server.")?;

	Ok(())
}
