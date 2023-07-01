use {
	crate::{ensure_map, ensure_server, update_player},
	color_eyre::{
		eyre::{bail as yeet, Context, ContextCompat},
		Result,
	},
	gokz_rs::{error::Error as GokzError, global_api},
	sqlx::PgPool,
	tokio::time::Duration,
	tracing::warn,
};

/// Time to wait between each API call
pub const DELAY: Duration = Duration::from_millis(727);

#[tracing::instrument(level = "TRACE", err(Debug))]
pub async fn fetch_records(
	start_id: Option<u32>,
	gokz_client: &gokz_rs::Client,
	pool: &PgPool,
) -> Result<()> {
	let mut record_id = match start_id {
		Some(record_id) => record_id,
		None => {
			let max_id: u32 = sqlx::query!("SELECT MAX(id) id FROM records")
				.fetch_one(pool)
				.await
				.context("Failed to fetch RecordID from database.")?
				.id
				.context("No records in database yet.")?
				.try_into()
				.context("Found negative RecordID in database.")?;

			max_id + 1
		}
	};

	loop {
		tokio::time::sleep(DELAY).await;

		let record = match global_api::get_record(record_id, gokz_client).await {
			Ok(record) => record,
			Err(error) => match error {
				// API is down or record does not exist (don't care)
				GokzError::Http {
					code, ..
				} if matches!(code.as_u16(), 500..=502) => {
					warn!(%record_id, "No new records... sleeping {DELAY:?}");
					continue;
				}

				error => yeet!("GlobalAPI request failed: {error:?}"),
			},
		};

		// Make sure map exists in the database
		ensure_map(record.map_id.into(), gokz_client, pool).await?;

		// Make sure server exists in the database
		ensure_server(record.server_id.into(), gokz_client, pool).await?;

		// Update player information
		update_player(record.steam_id, &record.player_name, gokz_client, pool).await?;

		let course_id = record.map_id as i32 * 100 + record.stage as i32;

		sqlx::query! {
			r#"
			INSERT INTO records
				(id, course_id, mode_id, player_id, server_id, time, teleports, created_on)
			VALUES
				($1, $2, $3, $4, $5, $6, $7, $8)
			"#,
			record.id as i32,
			course_id,
			record.mode as i16,
			record.steam_id.community_id() as i64,
			record.server_id as i16,
			record.time,
			record.teleports as i16,
			record.created_on,
		}
		.execute(pool)
		.await
		.context("Failed to insert record into database.")?;

		record_id += 1;
	}
}
