mod setup;

use {
	crate::setup::{Args, Config},
	axum::Server,
	clap::Parser,
	color_eyre::{eyre::Context, Result},
	schnose_api::{models::app_state::AppState, SchnoseAPI},
	sqlx::postgres::PgPoolOptions,
	std::sync::Arc,
	tracing::{debug, info, trace},
};

#[tokio::main]
async fn main() -> Result<()> {
	// Error handling
	color_eyre::install()?;

	// CLI arguments
	let args = Args::parse();

	// Logging
	crate::tracing_setup!(args.debug);

	let Config {
		database_url,
		mut ip_address,
	} = Config::load(&args).await.context("Failed to load config.")?;

	if args.public {
		ip_address.set_ip([0, 0, 0, 0].into());
	}

	if let Some(port) = args.port {
		ip_address.set_port(port);
	}

	debug!("Connecting to database...");

	let pool = PgPoolOptions::new()
		.min_connections(4)
		.max_connections(16)
		.connect(&database_url)
		.await
		.context("Failed to connect to database.")?;

	info!("Registering routes...");

	for route in SchnoseAPI::routes() {
		debug!(?route);
	}

	info!("SwaggerUI: {ip_address}/api/docs/swagger");
	info!("OpenAPI Spec: {ip_address}/api/docs/spec.json");

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
