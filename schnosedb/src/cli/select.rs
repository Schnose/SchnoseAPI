use {
	color_eyre::{eyre::Context, Result},
	serde::Serialize,
	sqlx::{MySql, Pool, QueryBuilder},
	tracing::info,
};

#[tracing::instrument(skip(database_connection))]
pub async fn select_rows<R>(
	table: &str,
	limit: Option<usize>,
	database_connection: &Pool<MySql>,
) -> Result<Vec<u8>>
where
	R: for<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> + Serialize + Send + Unpin,
{
	info!(
		"Selecting {limit} rows from `{table}`...",
		limit = match limit {
			Some(limit) => limit.to_string(),
			None => String::from("all"),
		}
	);

	let mut query = QueryBuilder::new(format!("SELECT * FROM {table}"));

	if let Some(limit) = limit {
		query
			.push(" LIMIT ")
			.push_bind(limit as u64);
	}

	let rows = query
		.build_query_as::<R>()
		.fetch_all(database_connection)
		.await
		.context("Failed to fetch rows from database.")?;

	serde_json::to_vec(&rows).context("Failed to serialize rows.")
}
