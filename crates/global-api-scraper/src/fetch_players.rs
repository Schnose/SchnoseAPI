use crate::FETCH_DELAY;

use {
	color_eyre::{eyre::Context, Result},
	gokz_rs::global_api,
	schnosedb::models::PlayerRow,
	sqlx::{MySql, Pool, QueryBuilder},
	tracing::warn,
};

pub async fn fetch_and_insert(
	start_offset: u32,
	gokz_client: &gokz_rs::Client,
	database_connection: &Pool<MySql>,
) -> Result<()> {
	let mut offset = start_offset;
	loop {
		let players = loop {
			if let Ok(players) = global_api::get_players(offset as i32, 500, gokz_client).await {
				break players;
			}

			warn!("[{offset}] No new players... sleeping for {}ms.", FETCH_DELAY.as_millis());
			std::thread::sleep(FETCH_DELAY);
		};

		let players = players
			.into_iter()
			.map(|player| PlayerRow {
				id: player.steam_id.as_id32(),
				name: player.name,
				is_banned: player.is_banned,
			})
			.collect::<Vec<_>>();

		let mut query = QueryBuilder::new(
			r#"
			INSERT INTO players
			  VALUES (id, name, is_banned)
			"#,
		);

		query.push_values(players, |mut query, player| {
			query
				.push_bind(player.id)
				.push_bind(player.name)
				.push_bind(player.is_banned as u8);
		});

		query
			.build()
			.execute(database_connection)
			.await
			.context("Failed to insert record into database.")?;

		match offset {
			500.. => offset -= 500,
			n @ 1.. => offset -= n,
			0 => {}
		};

		std::thread::sleep(FETCH_DELAY);
	}
}
