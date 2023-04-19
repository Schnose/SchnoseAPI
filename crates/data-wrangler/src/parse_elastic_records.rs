use {
	crate::Args,
	color_eyre::{
		eyre::{eyre, Context},
		Result,
	},
	elastic_scraper::elastic::ElasticRecord,
	schnosedb::models::{MapRow, ServerRow},
	sqlx::{MySql, Pool},
	tracing::info,
};

pub async fn parse(
	elastic_records: Vec<ElasticRecord>,
	database_connection: &Pool<MySql>,
	args: &Args,
) -> Result<()> {
	let mut records = Vec::new();

	let map_ids: Vec<MapRow> = sqlx::query_as("SELECT * FROM maps")
		.fetch_all(database_connection)
		.await
		.context("Failed to get maps")?;

	let server_ids: Vec<ServerRow> = sqlx::query_as("SELECT * FROM servers")
		.fetch_all(database_connection)
		.await
		.context("Failed to get servers")?;

	for (i, mut record) in elastic_records.into_iter().enumerate() {
		record.map_name = match record.map_name.as_str() {
			"kz_cyberspace_fix" => String::from("kz_cybersand"),
			"kz_hoist" => String::from("kz_hoist_fix"),
			"kz_gus" => String::from("kz_gus_sct2"),
			_ => record.map_name,
		};

		let map_id = map_ids
			.iter()
			.find_map(|map| {
				if map.name == record.map_name || map.name.contains(&record.map_name) {
					return Some(map.id);
				}
				None
			})
			.ok_or(eyre!("Failed to find `{}`", record.map_name))?;

		let server_id = server_ids
			.iter()
			.find_map(|server| {
				if server.name == record.server_name
					|| server
						.name
						.contains(&record.server_name)
				{
					return Some(server.id);
				}
				None
			})
			.unwrap_or(0);

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
		info!("[#{i}] Parsed record.");
	}

	let bytes = serde_json::to_vec(&records).context("Failed to serialize records.")?;
	if args.output_path.is_file() {
		std::fs::write(&args.output_path, bytes).context("Failed to write JSON to disk.")
	} else {
		std::fs::write(args.output_path.join("records.json"), bytes)
			.context("Failed to write JSON to disk.")
	}
}
