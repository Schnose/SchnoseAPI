use {
	crate::{Args, MapID, ServerID},
	color_eyre::{eyre::Context, Result},
	elastic_scraper::elastic::ElasticRecord,
	sqlx::{MySql, Pool},
};

pub async fn parse(
	elastic_records: Vec<ElasticRecord>,
	database_connection: &Pool<MySql>,
	args: &Args,
) -> Result<()> {
	let mut records = Vec::new();

	for record in elastic_records {
		let MapID(map_id) = sqlx::query_as("SELECT id FROM maps WHERE name = ?")
			.bind(&record.map_name)
			.fetch_one(database_connection)
			.await
			.context(format!("Failed to get map_id for `{}`", record.map_name))?;

		let ServerID(server_id) = sqlx::query_as("SELECT id FROM servers WHERE name = ?")
			.bind(&record.server_name)
			.fetch_one(database_connection)
			.await
			.context(format!("Failed to get server_id for `{}`", record.server_name))?;

		let record = schnosedb::models::RecordRow {
			id: record.id,
			course_id: (map_id as u32 * 1000) + record.stage as u32,
			mode_id: record.mode as u8,
			player_id: record.steam_id.as_id32(),
			server_id,
			time: record.time,
			teleports: record
				.teleports
				.try_into()
				.context("Teleports exceeded u16::MAX")?,
			created_on: record.created_on,
		};

		records.push(record);
	}

	let bytes = serde_json::to_vec(&records).context("Failed to serialize records.")?;
	std::fs::write(&args.output_path, bytes).context("Failed to write JSON to disk.")
}
