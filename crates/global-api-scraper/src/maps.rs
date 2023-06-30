use {
	color_eyre::{eyre::Context, Result},
	gokz_rs::{global_api, kzgo_api},
	sqlx::{PgPool, QueryBuilder},
	std::collections::HashMap,
	tracing::trace,
};

#[tracing::instrument(level = "TRACE", err(Debug))]
pub async fn fetch_maps(gokz_client: &gokz_rs::Client, pool: &PgPool) -> Result<usize> {
	let mut total = 0;

	let global_api_maps = global_api::get_maps(9999, gokz_client)
		.await
		.context("Failed to fetch maps from the GlobalAPI.")?;

	trace!(amount = %global_api_maps.len(), "Fetched maps from the GlobalAPI.");

	let mut kzgo_maps = kzgo_api::get_maps(gokz_client)
		.await
		.context("Failed to fetch maps from KZ:GO.")?
		.into_iter()
		.map(|map| (map.name.clone(), map))
		.collect::<HashMap<_, _>>();

	trace!(amount = %global_api_maps.len(), "Fetched maps from KZ:GO.");

	let maps = global_api_maps
		.into_iter()
		.map(|map| {
			let kzgo_map = kzgo_maps.remove(&map.name);
			(map, kzgo_map)
		})
		.collect::<Vec<_>>();

	trace!("Merged maps together.");
	total += maps.len();

	let courses = maps
		.iter()
		.flat_map(|(global_api_map, kzgo_map)| {
			let stages = kzgo_map.as_ref().map(|map| map.bonuses).unwrap_or(0);

			(0..=stages).map(|stage| (global_api_map.id, stage, global_api_map.difficulty))
		})
		.collect::<Vec<_>>();

	trace!("Processed courses.");

	let mut query = QueryBuilder::new(
		r#"
		INSERT INTO maps
			(id, name, global, workshop_id, filesize, created_on, updated_on)
		"#,
	);

	query.push_values(maps, |mut query, (global_api_map, kzgo_map)| {
		let workshop_id: Option<i64> = 'scope: {
			match global_api_map.workshop_url {
				None => {
					if let Some(map) = kzgo_map {
						if let Ok(id) = map.workshop_id.parse() {
							break 'scope Some(id);
						}
					}
				}
				Some(url) => {
					if let Some((_, id)) = url.rsplit_once('?') {
						if let Ok(id) = id.parse() {
							break 'scope Some(id);
						}
					}
				}
			};

			None
		};

		let filesize = global_api_map.filesize as i64;
		let filesize = (filesize > 0).then_some(filesize);

		query
			.push_bind(global_api_map.id as i16)
			.push_bind(global_api_map.name)
			.push_bind(global_api_map.validated)
			.push_bind(workshop_id)
			.push_bind(filesize)
			.push_bind(global_api_map.created_on)
			.push_bind(global_api_map.updated_on);
	});

	query.build().execute(pool).await.context("Failed to insert maps into database.")?;

	trace!("Inserted maps into database.");

	let mut query = QueryBuilder::new(
		r#"
		INSERT INTO courses
			(id, map_id, stage, tier)
		"#,
	);

	query.push_values(courses, |mut query, (map_id, stage, tier)| {
		let course_id = map_id as i32 * 100 + stage as i32;
		let tier = (stage == 0).then_some(tier as i16);

		query.push_bind(course_id).push_bind(map_id as i16).push_bind(stage as i16).push_bind(tier);
	});

	query.build().execute(pool).await.context("Failed to insert courses into database.")?;

	trace!("Inserted courses into database.");

	Ok(total)
}
