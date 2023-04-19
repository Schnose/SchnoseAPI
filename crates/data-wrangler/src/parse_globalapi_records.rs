use {
	crate::Args,
	color_eyre::{eyre::Context, Result},
	gokz_rs::global_api::Record,
	sqlx::types::chrono::{DateTime, Utc},
};

pub fn parse(elastic_records: Vec<Record>, args: &Args) -> Result<()> {
	let mut records = Vec::new();

	for record in elastic_records {
		let record = schnosedb::models::RecordRow {
			id: record.id,
			course_id: (record.map_id as u32 * 1000) + record.stage as u32,
			mode_id: record.mode as u8,
			player_id: record.steam_id.as_id32(),
			server_id: record.server_id,
			time: record.time,
			teleports: record
				.teleports
				.try_into()
				.context("Teleports exceeded u16::MAX")?,
			created_on: DateTime::<Utc>::from_utc(record.created_on, Utc),
		};

		records.push(record);
	}

	let bytes = serde_json::to_vec(&records).context("Failed to serialize records.")?;
	std::fs::write(&args.output_path, bytes).context("Failed to write JSON to disk.")
}
