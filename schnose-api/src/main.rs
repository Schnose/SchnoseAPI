mod setup;

pub use schnose_api::models::app_state::AppState;
use {
	crate::setup::{Args, Config},
	axum::Server,
	clap::Parser,
	color_eyre::{eyre::Context, Result},
	schnose_api::SchnoseAPI,
	sqlx::postgres::PgPoolOptions,
	std::sync::Arc,
	tracing::{info, trace},
	tracing_subscriber::util::SubscriberInitExt,
};

#[tokio::main]
async fn main() -> Result<()> {
	// Error handling
	color_eyre::install()?;

	// CLI arguments
	let args = Args::parse();

	// Logging
	if args.debug {
		just_trace::registry!(verbose).init();
	}

	let Config {
		database_url,
		ip_address,
	} = Config::load(&args).await.context("Failed to load config.")?;

	trace!("Connecting to database...");

	let pool = PgPoolOptions::new()
		.connect(&database_url)
		.await
		.context("Failed to connect to database.")?;

	trace!("Registering routes...");

	for route in SchnoseAPI::routes() {
		trace!(?route);
	}

	trace!("SwaggerUI: {ip_address}/docs/swagger");
	trace!("OpenAPI Spec: {ip_address}/docs/spec.json");

	let app_state = Arc::new(AppState {
		pool,
	});

	let router = schnose_api::routes::router(app_state);

	trace!("Binding to {ip_address}...");
	let server = Server::bind(&ip_address).serve(router.into_make_service());

	info!("Listening on {}.", server.local_addr());

	server.await.context("Failed to run API.")?;

	Ok(())
}
