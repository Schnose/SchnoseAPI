use {
	color_eyre::{eyre::Context, Result},
	gokz_rs::{
		global_api, kzgo_api,
		types::{MapIdentifier, PlayerIdentifier, ServerIdentifier, SteamID},
	},
	sqlx::{PgPool, QueryBuilder},
	tracing::trace,
};

mod players;
pub use players::fetch_players;

mod maps;
pub use maps::fetch_maps;

mod servers;
pub use servers::fetch_servers;

mod records;
pub use records::fetch_records;

mod mappers;
pub use mappers::fetch_mappers;

mod filters;
pub use filters::fetch_filters;

#[tracing::instrument(level = "TRACE", skip(gokz_client, pool))]
pub async fn ensure_player(
	player: PlayerIdentifier,
	gokz_client: &gokz_rs::Client,
	pool: &PgPool,
) -> Result<SteamID> {
	let steam_id = match player {
		PlayerIdentifier::SteamID(ref steam_id) => {
			sqlx::query!("SELECT id FROM players WHERE id = $1", steam_id.community_id() as i64)
				.fetch_optional(pool)
				.await
				.context("Failed to fetch player from database")?
				.map(|row| row.id)
		}

		PlayerIdentifier::Name(ref name) => {
			sqlx::query!("SELECT id FROM players WHERE name LIKE $1", format!("%{name}%"))
				.fetch_optional(pool)
				.await
				.context("Failed to fetch player from database")?
				.map(|row| row.id)
		}
	};

	if let Some(steam_id) = steam_id {
		return u32::try_from(steam_id)
			.context("Found invalid SteamID in database.")?
			.try_into()
			.context("Found invalid SteamID in database.");
	}

	let player = global_api::get_player(player, gokz_client)
		.await
		.context("Failed to fetch player from the GlobalAPI.")?;

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
	.await
	.context("Failed to insert player into database.")?;

	Ok(player.steam_id)
}

#[tracing::instrument(level = "TRACE", skip(gokz_client, pool))]
pub async fn ensure_map(
	map: MapIdentifier,
	gokz_client: &gokz_rs::Client,
	pool: &PgPool,
) -> Result<(u16, String)> {
	let db_map = match map {
		MapIdentifier::Id(map_id) => {
			sqlx::query!("SELECT id, name FROM maps WHERE id = $1", map_id as i16)
				.fetch_optional(pool)
				.await
				.context("Failed to fetch map from database")?
				.map(|row| (row.id, row.name))
		}

		MapIdentifier::Name(ref name) => {
			sqlx::query!("SELECT id, name FROM maps WHERE name LIKE $1", format!("%{name}%"))
				.fetch_optional(pool)
				.await
				.context("Failed to fetch map from database")?
				.map(|row| (row.id, row.name))
		}
	};

	if let Some((id, name)) = db_map {
		return Ok((id as u16, name));
	}

	let global_api_map = global_api::get_map(map, gokz_client)
		.await
		.context("Failed to fetch map from the GlobalAPI.")?;

	let kzgo_map = kzgo_api::get_map(&global_api_map.name, gokz_client)
		.await
		.context("Failed to fetch map from KZ:GO.")?;

	let courses =
		(0..=kzgo_map.bonuses).map(|stage| (global_api_map.id, stage, global_api_map.difficulty));

	let workshop_id: Option<i64> = 'scope: {
		match global_api_map.workshop_url {
			None => {
				if let Ok(id) = kzgo_map.workshop_id.parse() {
					break 'scope Some(id);
				}
			}
			Some(url) => {
				if let Some((_, id)) = url.rsplit_once('?') {
					if let Ok(id) = id.parse() {
						break 'scope Some(id);
					}
				}
			}
		};

		None
	};

	let filesize = global_api_map.filesize as i64;
	let filesize = (filesize > 0).then_some(filesize);

	sqlx::query! {
		r#"
		INSERT INTO maps
			(id, name, global, workshop_id, filesize, created_on, updated_on)
		VALUES
			($1, $2, $3, $4, $5, $6, $7)
		"#,
		global_api_map.id as i16,
		global_api_map.name,
		global_api_map.validated,
		workshop_id,
		filesize,
		global_api_map.created_on,
		global_api_map.updated_on,
	}
	.execute(pool)
	.await
	.context("Failed to insert map into database.")?;

	trace!("Inserted maps into database.");

	let mut query = QueryBuilder::new(
		r#"
		INSERT INTO courses
			(id, map_id, stage, tier)
		VALUES
			($1, $2, $3, $4)
		"#,
	);

	query.push_values(courses, |mut query, (map_id, stage, tier)| {
		let course_id = map_id as i32 * 100 + stage as i32;
		let tier = (stage == 0).then_some(tier as i16);

		query.push_bind(course_id).push_bind(map_id as i16).push_bind(stage as i16).push_bind(tier);
	});

	query.build().execute(pool).await.context("Failed to insert courses into database.")?;

	trace!("Inserted courses into database.");

	Ok((global_api_map.id, global_api_map.name))
}

#[tracing::instrument(level = "TRACE", skip(gokz_client, pool))]
pub async fn ensure_server(
	server: ServerIdentifier,
	gokz_client: &gokz_rs::Client,
	pool: &PgPool,
) -> Result<(u16, String)> {
	let db_server = match server {
		ServerIdentifier::Id(server_id) => {
			sqlx::query!("SELECT id, name FROM servers WHERE id = $1", server_id as i16)
				.fetch_optional(pool)
				.await
				.context("Failed to fetch server from database")?
				.map(|row| (row.id, row.name))
		}

		ServerIdentifier::Name(ref name) => {
			sqlx::query!("SELECT id, name FROM servers WHERE name LIKE $1", format!("%{name}%"))
				.fetch_optional(pool)
				.await
				.context("Failed to fetch server from database")?
				.map(|row| (row.id, row.name))
		}
	};

	if let Some((id, name)) = db_server {
		return Ok((id as u16, name));
	}

	let server = global_api::get_server(server, gokz_client)
		.await
		.context("Failed to fetch server from GlobalAPI.")?;

	ensure_player(server.owner_id.into(), gokz_client, pool).await?;

	sqlx::query! {
		r#"
		INSERT INTO servers
			(id, name, owned_by)
		VALUES
			($1, $2, $3)
		"#,
		server.id as i16,
		server.name,
		server.owner_id.community_id() as i64,
	}
	.execute(pool)
	.await
	.context("Failed to insert server into database.")?;

	trace!("Inserted servers into database.");

	Ok((server.id, server.name))
}

#[tracing::instrument(level = "TRACE", skip(pool))]
pub async fn update_player(steam_id: SteamID, player_name: &str, pool: &PgPool) -> Result<()> {
	sqlx::query! {
		r#"
		INSERT INTO players
			(id, name, is_banned)
		VALUES
			($1, $2, false)
		ON CONFLICT (id) DO UPDATE
		SET name = $2, is_banned = false
		"#,
		steam_id.community_id() as i64,
		player_name,
	}
	.execute(pool)
	.await
	.context("Failed to update player information.")?;

	Ok(())
}
