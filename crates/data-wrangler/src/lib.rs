use {
	color_eyre::{eyre::Context, Result},
	gokz_rs::{global_api, kzgo_api, types::Tier},
	sqlx::{PgPool, QueryBuilder},
	tracing::trace,
	zer0k_elastic_scraper::elastic,
};

#[tracing::instrument(level = "INFO", skip(pool), err(Debug))]
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
					.push_bind(steam_id.community_id() as i32)
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

#[tracing::instrument(level = "INFO", skip(pool), err(Debug))]
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
			let approved_by = approved_by.map(|approved_by| approved_by.community_id() as i32);

			query
				.push_bind(workshop_id as i32)
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

#[tracing::instrument(level = "INFO", skip(pool), err(Debug))]
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

#[tracing::instrument(level = "INFO", skip(pool), err(Debug))]
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
			query.push_bind(id as i16).push_bind(name).push_bind(owner_id.community_id() as i32);
		},
	);

	trace!("building query");
	query.build().execute(&mut transaction).await.context("Failed to execute query.")?;

	trace!("committing query");
	transaction.commit().await.context("Failed to commit SQL transaction.")?;

	Ok(processed)
}

#[tracing::instrument(level = "INFO", skip(pool), err(Debug))]
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
				.push_bind(steam_id.community_id() as i32)
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

#[tracing::instrument(level = "INFO", skip(pool), err(Debug))]
pub async fn insert_elastic_records(records: Vec<elastic::Record>, pool: &PgPool) -> Result<usize> {
	let mut query = QueryBuilder::new(
		r#"
		INSERT INTO records
			(id, course_id, mode_id, player_id, server_id, time, teleports, created_on)
		"#,
	);

	let processed = records.len();

	trace!(amount = %records.len(), "processing records");

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

	let mut transaction = pool.begin().await.context("Failed to start SQL transaction.")?;

	query.push_values(
		records,
		|mut query,
		 elastic::Record {
		     id,
		     map_name,
		     stage,
		     mode,
		     steam_id,
		     teleports,
		     time,
		     server_name,
		     created_on,
		     ..
		 }| {
			let Some(map) = maps.iter().find(|map| {
				map.name == map_name
					|| map.name.contains(&map_name)
			}) else {
				return;
			};

			let course_id = map.id as i32 * 100 + stage as i32;

			let Some(server) = servers.iter().find(|server| {
				server.name == server_name
					|| server.name.contains(&server_name)
			}) else {
				return;
			};

			query
				.push_bind(id as i32)
				.push_bind(course_id)
				.push_bind(mode as i16)
				.push_bind(steam_id.community_id() as i32)
				.push_bind(server.id)
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
