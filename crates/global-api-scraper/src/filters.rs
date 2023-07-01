use {
	color_eyre::{eyre::Context, Result},
	gokz_rs::kzgo_api,
	sqlx::{PgPool, QueryBuilder},
};

#[tracing::instrument(level = "TRACE", skip(gokz_client, pool), err(Debug))]
pub async fn fetch_filters(gokz_client: &gokz_rs::Client, pool: &PgPool) -> Result<usize> {
	let mut filters: Vec<(i32, i16)> = Vec::new();

	for map in kzgo_api::get_maps(gokz_client).await.context("Failed to fetch maps from KZ:GO.")? {
		let course_id = map.id as i32 * 100;

		if !map.name.starts_with("skz_") && !map.name.starts_with("vnl_") {
			filters.push((course_id, 200));
		}

		if map.skz {
			filters.push((course_id, 201));
		}

		if map.vnl {
			filters.push((course_id, 202));
		}
	}

	let total = filters.len();

	let mut query = QueryBuilder::new(
		r#"
		INSERT INTO filters
			(course_id, mode_id)
		"#,
	);

	query.push_values(filters, |mut query, (course_id, mode_id)| {
		query.push_bind(course_id).push_bind(mode_id);
	});

	query.build().execute(pool).await.context("Failed to insert filters into database.")?;

	Ok(total)
}
