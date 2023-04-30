use {
	global_api_scraper::{fetch_records, RecordID},
	shuttle_runtime::async_trait,
	shuttle_secrets::SecretStore,
	sqlx::{mysql::MySqlPoolOptions, MySql, Pool},
	std::{net::SocketAddr, sync::Arc, time::Instant},
	tracing::info,
};

#[derive(Debug, Clone)]
pub struct State {
	database_connection: Arc<Pool<MySql>>,
	gokz_client: Arc<gokz_rs::Client>,
}

pub type ShuttleResult = Result<State, shuttle_service::Error>;

#[async_trait]
impl shuttle_service::Service for State {
	async fn bind(self, _: SocketAddr) -> Result<(), shuttle_service::Error> {
		let start_id = sqlx::query_as::<_, RecordID>("SELECT MAX(id) AS id FROM records")
			.fetch_one(&*self.database_connection)
			.await
			.expect("Failed to get initial record id from database.")
			.0 + 1;

		let start = Instant::now();
		fetch_records::fetch_and_insert(start_id, &self.gokz_client, &self.database_connection)
			.await
			.expect("Failed to fetch records.");
		info!("Done. (took {:?})", start.elapsed());

		Ok(())
	}
}

#[shuttle_runtime::main]
async fn scraper(#[shuttle_secrets::Secrets] secrets: SecretStore) -> ShuttleResult {
	let database_connection = MySqlPoolOptions::new()
		.connect(
			&secrets
				.get("CONNECTION_STRING")
				.expect("Missing `CONNECTION_STRING` secret."),
		)
		.await
		.expect("Failed to establish database connection.");

	let gokz_client = gokz_rs::Client::new();

	Ok(State {
		database_connection: Arc::new(database_connection),
		gokz_client: Arc::new(gokz_client),
	})
}
