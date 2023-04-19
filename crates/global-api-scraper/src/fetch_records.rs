use {
	crate::FETCH_DELAY,
	color_eyre::{eyre::Context, Result},
	gokz_rs::global_api,
	schnosedb::models::RecordRow,
	sqlx::{
		types::chrono::{DateTime, Utc},
		MySql, Pool, QueryBuilder,
	},
	tracing::warn,
};

pub async fn fetch_and_insert(
	start_id: u32,
	gokz_client: &gokz_rs::Client,
	database_connection: &Pool<MySql>,
) -> Result<()> {
	for id in start_id.. {
		let record = loop {
			if let Ok(record) = global_api::get_record(id, gokz_client).await {
				break record;
			}

			warn!("[{id}] No new records... sleeping for {}ms.", FETCH_DELAY.as_millis());
			std::thread::sleep(FETCH_DELAY);
		};

		let record = RecordRow {
			id: record.id,
			course_id: (record.map_id as u32 * 1000) + record.stage as u32,
			mode_id: record.mode as u8,
			player_id: record.steam_id.as_id32(),
			server_id: record.server_id,
			time: record.time,
			teleports: record
				.teleports
				.try_into()
				.context("`teleports` is out of bounds")?,
			created_on: DateTime::<Utc>::from_utc(record.created_on, Utc),
		};

		let mut query = QueryBuilder::new(
			r#"
			INSERT INTO records
			  VALUES (id, course_id, mode_id, player_id, server_id, time, teleports, created_on)
			"#,
		);

		query.push_values([record], |mut query, record| {
			query
				.push_bind(record.id)
				.push_bind(record.course_id)
				.push_bind(record.mode_id)
				.push_bind(record.player_id)
				.push_bind(record.server_id)
				.push_bind(record.time)
				.push_bind(record.teleports)
				.push_bind(record.created_on);
		});

		query
			.build()
			.execute(database_connection)
			.await
			.context("Failed to insert record into database.")?;

		std::thread::sleep(FETCH_DELAY);
	}

	Ok(())
}
