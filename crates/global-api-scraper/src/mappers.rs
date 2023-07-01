use {
	crate::ensure_player,
	color_eyre::{eyre::Context, Result},
	gokz_rs::kzgo_api,
	sqlx::{PgPool, QueryBuilder},
	std::collections::HashSet,
};

#[tracing::instrument(level = "TRACE", skip(gokz_client, pool), err(Debug))]
pub async fn fetch_mappers(gokz_client: &gokz_rs::Client, pool: &PgPool) -> Result<usize> {
	let mut total = 0;

	let mappers = kzgo_api::get_maps(gokz_client)
		.await
		.context("Failed to fetch maps from KZ:GO.")?
		.into_iter()
		.flat_map(|map| map.mapper_ids.into_iter().map(move |steam_id| (steam_id, map.id)))
		.collect::<HashSet<_>>();

	for &(mapper, _) in &mappers {
		// Make sure mapper is in the database
		ensure_player(mapper.into(), gokz_client, pool).await?;
		total += 1;
	}

	let mut query = QueryBuilder::new(
		r#"
		INSERT INTO mappers
			(player_id, map_id)
		"#,
	);

	query.push_values(mappers, |mut query, (steam_id, map_id)| {
		query.push_bind(steam_id.community_id() as i64).push_bind(map_id as i16);
	});

	query.build().execute(pool).await.context("Failed to insert mappers into database.")?;

	Ok(total)
}
