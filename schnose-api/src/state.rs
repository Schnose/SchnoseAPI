use {
	color_eyre::{eyre::Context, Result},
	sqlx::{mysql::MySqlPoolOptions, MySql, Pool},
	std::sync::Arc,
};

#[derive(Debug, Clone)]
pub struct APIState {
	pub database_connection: Arc<Pool<MySql>>,
}

impl APIState {
	#[tracing::instrument(skip(connection_string))]
	pub async fn new(connection_string: &str) -> Result<Self> {
		Ok(Self {
			database_connection: Arc::new(
				MySqlPoolOptions::new()
					.connect(connection_string)
					.await
					.context("Failed to establish database connection.")?,
			),
		})
	}

	pub fn db(&self) -> &Pool<MySql> {
		&self.database_connection
	}
}
