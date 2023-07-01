use {
	color_eyre::{eyre::Context, Result},
	gokz_rs::global_api,
	sqlx::{PgPool, QueryBuilder},
	tracing::trace,
};

#[tracing::instrument(level = "TRACE", skip(gokz_client, pool), err(Debug))]
pub async fn fetch_servers(gokz_client: &gokz_rs::Client, pool: &PgPool) -> Result<usize> {
	let mut total = 0;

	let servers = global_api::get_servers(9999, gokz_client)
		.await
		.context("Failed to fetch servers from the GlobalAPI.")?;

	trace!(amount = %servers.len(), "Fetched servers from the GlobalAPI.");

	total += servers.len();

	for server in &servers {
		// Make sure all server owners are in the database.
		crate::ensure_player(server.owner_id.into(), gokz_client, pool).await?;
	}

	let mut query = QueryBuilder::new(
		r#"
		INSERT INTO servers
			(id, name, owned_by)
		"#,
	);

	query.push_values(servers, |mut query, server| {
		query
			.push_bind(server.id as i16)
			.push_bind(server.name)
			.push_bind(server.owner_id.community_id() as i64);
	});

	query.build().execute(pool).await.context("Failed to insert servers into database.")?;

	trace!("Inserted servers into database.");

	Ok(total)
}
