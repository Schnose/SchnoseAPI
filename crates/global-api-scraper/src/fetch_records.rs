use {
	crate::FETCH_DELAY,
	color_eyre::{eyre::Context, Result},
	gokz_rs::{global_api, ServerIdentifier},
	schnosedb::models::{PlayerRow, RecordRow, ServerRow},
	sqlx::{
		types::chrono::{DateTime, Utc},
		MySql, Pool, QueryBuilder,
	},
	tracing::warn,
};

const ZPAMM: u32 = 17690692;

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

		let player_id = match sqlx::query_as::<_, PlayerRow>("SELECT * FROM players WHERE id = ?")
			.bind(record.steam_id.as_id32())
			.fetch_optional(database_connection)
			.await
			.context("Failed to fetch player from database.")?
		{
			Some(player) => {
				// Player is banned but we just got a record, which means they got unbanned.
				if player.is_banned {
					sqlx::query("UPDATE players SET is_banned = 0 WHERE id = ?")
						.bind(player.id)
						.execute(database_connection)
						.await
						.context("Failed to updated player's banned status.")?;
				}

				// Player changed their name.
				if player.name != record.player_name {
					sqlx::query("UPDATE players SET name = ? WHERE id = ?")
						.bind(player.name)
						.bind(player.id)
						.execute(database_connection)
						.await
						.context("Failed to updated player's name.")?;
				}
				player.id
			}
			None => {
				sqlx::query("INSERT INTO players (id, name, is_banned) VALUES (?, ?, ?)")
					.bind(record.steam_id.as_id32())
					.bind(record.player_name)
					.bind(0)
					.execute(database_connection)
					.await
					.context("Failed to insert new player.")?;
				record.steam_id.as_id32()
			}
		};

		let server_id = match sqlx::query_as::<_, ServerRow>("SELECT * FROM servers WHERE id = ?")
			.bind(record.server_id)
			.fetch_optional(database_connection)
			.await
			.context("Failed to fetch server from database.")?
		{
			Some(server) => {
				// Server changed its name.
				if server.name != record.server_name {
					sqlx::query("UPDATE servers SET name = ? WHERE id = ?")
						.bind(server.name)
						.bind(server.id)
						.execute(database_connection)
						.await
						.context("Failed to updated server's name.")?;
				}
				server.id
			}
			None => {
				let server =
					global_api::get_server(&ServerIdentifier::ID(record.server_id), gokz_client)
						.await
						.context("Failed to fetch server from GlobalAPI.")?;

				sqlx::query(
					"INSERT INTO servers (id, name, owned_by, approved_by) VALUES (?, ?, ?, ?)",
				)
				.bind(server.id)
				.bind(server.name)
				.bind(server.owner_steamid.as_id32())
				.bind(ZPAMM)
				.execute(database_connection)
				.await
				.context("Failed to insert new server.")?;
				server.id
			}
		};

		let record = RecordRow {
			id: record.id,
			course_id: (record.map_id as u32 * 1000) + record.stage as u32,
			mode_id: record.mode as u8,
			player_id,
			server_id,
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
