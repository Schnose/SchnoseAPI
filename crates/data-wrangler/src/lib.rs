use {
	color_eyre::{eyre::Context, Result},
	gokz_rs::{global_api, kzgo_api, types::Tier},
	sqlx::{PgPool, QueryBuilder},
	std::{collections::HashSet, time::Duration},
	tracing::{trace, warn},
	zer0k_elastic_scraper::elastic,
};

#[tracing::instrument(level = "INFO", skip(players, pool))]
pub async fn insert_players(players: Vec<global_api::Player>, pool: &PgPool) -> Result<usize> {
	let mut query = QueryBuilder::new(
		r#"
		INSERT INTO players
			(id, name, is_banned)
		"#,
	);

	let processed = players.len();

	for players in players.chunks(1000) {
		trace!(size = %players.len(), "processing chunk of players");

		let mut transaction = pool.begin().await.context("Failed to start SQL transaction.")?;

		query.push_values(
			players,
			|mut query,
			 global_api::Player {
			     name,
			     steam_id,
			     is_banned,
			 }| {
				query
					.push_bind(name)
					.push_bind(steam_id.community_id() as i64)
					.push_bind(is_banned);
			},
		);

		trace!("building query");
		query.build().execute(&mut transaction).await.context("Failed to execute query.")?;

		trace!("committing query");
		transaction.commit().await.context("Failed to commit SQL transaction.")?;

		query.reset();
	}

	Ok(processed)
}

#[tracing::instrument(level = "INFO", skip(maps, pool))]
pub async fn insert_maps(
	maps: Vec<(global_api::Map, kzgo_api::Map)>,
	pool: &PgPool,
) -> Result<usize> {
	let mut query = QueryBuilder::new(
		r#"
		INSERT INTO maps
			(id, name, ranked, workshop_id, filesize, approved_by, created_on, updated_on)
		"#,
	);

	let mut processed = maps.len();
	let courses = maps
		.iter()
		.map(|(global_api, kzgo)| (global_api.id, kzgo.bonuses + 1, global_api.difficulty))
		.collect::<Vec<_>>();

	trace!(amount = %maps.len(), "processing maps");

	let mut transaction = pool.begin().await.context("Failed to start SQL transaction.")?;

	query.push_values(
		maps,
		|mut query,
		 (
			global_api::Map {
				id,
				name,
				validated,
				filesize,
				approved_by,
				created_on,
				updated_on,
				..
			},
			kzgo_api::Map {
				workshop_id, ..
			},
		)| {
			query.push_bind(id as i16).push_bind(name).push_bind(validated);

			let filesize = (filesize > 0).then_some(filesize as i64);
			let approved_by = approved_by.map(|approved_by| approved_by.community_id() as i64);

			query
				.push_bind(workshop_id.parse::<i64>().ok())
				.push_bind(filesize)
				.push_bind(approved_by)
				.push_bind(created_on)
				.push_bind(updated_on);
		},
	);

	trace!("building query");
	query.build().execute(&mut transaction).await.context("Failed to execute query.")?;

	trace!("committing query");
	transaction.commit().await.context("Failed to commit SQL transaction.")?;

	processed += insert_courses(courses, pool).await.context("Failed to insert courses.")?;

	Ok(processed)
}

#[tracing::instrument(level = "INFO", skip(courses, pool))]
pub async fn insert_courses(courses: Vec<(u16, u8, Tier)>, pool: &PgPool) -> Result<usize> {
	let mut query = QueryBuilder::new(
		r#"
		INSERT INTO courses
			(id, map_id, stage, tier)
		"#,
	);

	let processed = courses.iter().map(|&(_, stages, _)| stages as usize).sum::<usize>();

	trace!(amount = %processed, "processing courses");

	let mut transaction = pool.begin().await.context("Failed to start SQL transaction.")?;

	query.push_values(courses, |mut query, (map_id, stage, tier)| {
		let course_id = map_id as i32 * 100 + stage as i32;

		let tier = (stage == 0).then_some(tier as i16);

		query.push_bind(course_id).push_bind(map_id as i16).push_bind(stage as i16).push_bind(tier);
	});

	trace!("building query");
	query.build().execute(&mut transaction).await.context("Failed to execute query.")?;

	trace!("committing query");
	transaction.commit().await.context("Failed to commit SQL transaction.")?;

	Ok(processed)
}

#[tracing::instrument(level = "INFO", skip(servers, pool))]
pub async fn insert_servers(servers: Vec<global_api::Server>, pool: &PgPool) -> Result<usize> {
	let mut query = QueryBuilder::new(
		r#"
		INSERT INTO servers
			(id, name, owned_by)
		"#,
	);

	let processed = servers.len();

	trace!(amount = %servers.len(), "processing servers");

	let mut transaction = pool.begin().await.context("Failed to start SQL transaction.")?;

	query.push_values(
		servers,
		|mut query,
		 global_api::Server {
		     id,
		     name,
		     owner_id,
		     ..
		 }| {
			query.push_bind(id as i16).push_bind(name).push_bind(owner_id.community_id() as i64);
		},
	);

	trace!("building query");
	query.build().execute(&mut transaction).await.context("Failed to execute query.")?;

	trace!("committing query");
	transaction.commit().await.context("Failed to commit SQL transaction.")?;

	Ok(processed)
}

#[tracing::instrument(level = "INFO", skip(records, pool))]
pub async fn insert_records(records: Vec<global_api::Record>, pool: &PgPool) -> Result<usize> {
	let mut query = QueryBuilder::new(
		r#"
		INSERT INTO records
			(id, course_id, mode_id, player_id, server_id, time, teleports, created_on)
		"#,
	);

	let processed = records.len();

	trace!(amount = %records.len(), "processing records");

	let mut transaction = pool.begin().await.context("Failed to start SQL transaction.")?;

	query.push_values(
		records,
		|mut query,
		 global_api::Record {
		     id,
		     map_id,
		     steam_id,
		     server_id,
		     stage,
		     mode,
		     teleports,
		     time,
		     created_on,
		     ..
		 }| {
			let course_id = map_id as i32 * 100 + stage as i32;

			query
				.push_bind(id as i32)
				.push_bind(course_id)
				.push_bind(mode as i16)
				.push_bind(steam_id.community_id() as i64)
				.push_bind(server_id as i16)
				.push_bind(time)
				.push_bind(teleports as i16)
				.push_bind(created_on);
		},
	);

	trace!("building query");
	query.build().execute(&mut transaction).await.context("Failed to execute query.")?;

	trace!("committing query");
	transaction.commit().await.context("Failed to commit SQL transaction.")?;

	Ok(processed)
}

#[tracing::instrument(
	level = "TRACE",
	skip(records, gokz_client, pool),
	fields(records = %records.len())
)]
pub async fn insert_elastic_records(
	records: Vec<elastic::Record>,
	gokz_client: &gokz_rs::Client,
	pool: &PgPool,
) -> Result<usize> {
	let mut query = QueryBuilder::new(
		r#"
		INSERT INTO records
			(id, course_id, mode_id, player_id, server_id, time, teleports, created_on)
		"#,
	);

	let maps = sqlx::query! {
		"SELECT * FROM maps"
	}
	.fetch_all(pool)
	.await
	.context("Failed to fetch maps")?;

	let servers = sqlx::query! {
		"SELECT * FROM servers"
	}
	.fetch_all(pool)
	.await
	.context("Failed to fetch servers")?;

	let mut valid = Vec::with_capacity(records.len());
	let mut player_cache = HashSet::new();
	let mut course_cache = HashSet::new();
	let mut bad_courses = HashSet::new();

	for record in records {
		if !player_cache.contains(&record.steam_id)
			&& sqlx::query! {
				"SELECT * FROM players WHERE id = $1",
				record.steam_id.community_id() as i64
			}
			.fetch_one(pool)
			.await
			.is_err()
		{
			warn!(name = %record.player_name, steam_id = %record.steam_id, "missing player, fetching...");
			let player = global_api::get_player(record.steam_id, gokz_client).await?;
			sqlx::query! {
				r#"
				INSERT INTO players
					(id, name, is_banned)
				VALUES
					($1, $2, $3)
				"#,
				player.steam_id.community_id() as i64,
				player.name,
				player.is_banned,
			}
			.execute(pool)
			.await?;

			tokio::time::sleep(Duration::from_millis(727)).await;

			player_cache.insert(record.steam_id);
		}

		let Some(map) = maps.iter().find(|map| {
			map.name == record.map_name
			|| map.name.contains(&record.map_name)
		}) else {
			continue;
		};

		let course_id = map.id as i32 * 100 + record.stage as i32;

		if !course_cache.contains(&course_id)
			&& (bad_courses.contains(&course_id)
				|| sqlx::query!("SELECT * FROM courses WHERE id = $1", course_id)
					.fetch_one(pool)
					.await
					.is_err())
		{
			bad_courses.insert(course_id);
			continue;
		}

		course_cache.insert(course_id);

		let Some(server) = servers.iter().find(|server| {
			server.name == record.server_name
			|| server.name.contains(&record.server_name)
		}) else {
			continue;
		};

		valid.push((server, course_id, record));
	}

	let processed = valid.len();

	trace!(amount = %valid.len(), "processing records");

	let mut transaction = pool.begin().await.context("Failed to start SQL transaction.")?;

	query.push_values(
		valid,
		|mut query,
		 (
			server,
			course_id,
			elastic::Record {
				id,
				mode,
				steam_id,
				teleports,
				time,
				created_on,
				..
			},
		)| {
			query
				.push_bind(id as i32)
				.push_bind(course_id)
				.push_bind(mode as i16)
				.push_bind(steam_id.community_id() as i64)
				.push_bind(server.id)
				.push_bind(time)
				.push_bind(teleports as i16)
				.push_bind(created_on);
		},
	);

	query.build().execute(&mut transaction).await.context("Failed to execute query.")?;

	trace!("committing query");
	transaction.commit().await.context("Failed to commit SQL transaction.")?;

	Ok(processed)
}
