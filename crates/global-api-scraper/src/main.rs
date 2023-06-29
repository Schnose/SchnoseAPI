mod args;
mod config;

use {
	crate::{args::Args, config::Config},
	args::Data,
	chrono::{DateTime, NaiveDateTime, Utc},
	clap::Parser,
	color_eyre::{
		eyre::{bail as yeet, eyre, Context},
		Result,
	},
	gokz_rs::{global_api, kzgo_api, prelude::Error as GokzError},
	sqlx::{postgres::PgPoolOptions, PgPool, QueryBuilder},
	std::{
		collections::{HashMap, HashSet},
		time::{Duration, Instant},
	},
	tracing::{error, info, trace, warn},
	tracing_subscriber::util::SubscriberInitExt,
};

#[tokio::main]
async fn main() -> Result<()> {
	// Error handling
	color_eyre::install()?;

	// CLI arguments
	let args = Args::parse();

	// Logging
	just_trace::registry!().init();

	let config = Config::load(&args)?;

	let start = Instant::now();

	let gokz_client = gokz_rs::Client::new();

	let pool = PgPoolOptions::new()
		.connect(&config.database_url)
		.await
		.context("Failed to establish database connection.")?;

	match args.data {
		Data::Players {
			start_id,
			backwards,
			chunk_size,
		} => {
			let mut params = global_api::players::Params {
				offset: Some(start_id as i32),
				limit: Some(chunk_size as u32),
				..Default::default()
			};

			loop {
				let Ok(players) = global_api::players::root(&params, &gokz_client).await else {
					info!("Done. (no players anymore)");
					break;
				};

				if players.is_empty() {
					info!("Done. (no players anymore)");
					break;
				}

				trace!(amount = %players.len(), "fetched players");

				let mut query = QueryBuilder::new(
					r#"
					INSERT INTO players
						(id, name, is_banned)
					"#,
				);

				let mut transaction = pool.begin().await.context("Failed to start transaction.")?;

				query.push_values(
					players,
					|mut query,
					 global_api::Player {
					     name,
					     steam_id,
					     is_banned,
					 }| {
						query
							.push_bind(steam_id.community_id() as i64)
							.push_bind(name)
							.push_bind(is_banned);
					},
				);

				query.push(" ON CONFLICT DO NOTHING ");

				query
					.build()
					.execute(&mut transaction)
					.await
					.context("Failed to insert players into database.")?;

				transaction.commit().await.context("Failed to commit transaction.")?;

				info!("inserted players");

				if let Some(offset) = params.offset.as_mut() {
					if backwards {
						*offset += chunk_size as i32;
					} else {
						*offset -= chunk_size as i32;
					}
				}
			}
		}

		Data::Maps => {
			let global_api_maps = global_api::get_maps(9999, &gokz_client)
				.await
				.context("Failed to fetch maps from the GlobalAPI")?;

			let mut kzgo_maps = kzgo_api::get_maps(&gokz_client)
				.await
				.context("Failed to fetch maps from KZ:GO")?
				.into_iter()
				.map(|map| (map.name.clone(), map))
				.collect::<HashMap<_, _>>();

			let maps = global_api_maps
				.into_iter()
				.map(|global_api_map| {
					let kzgo_map = kzgo_maps.remove(&global_api_map.name);
					(global_api_map, kzgo_map)
				})
				.collect::<Vec<_>>();

			let courses = maps
				.iter()
				.flat_map(|(global_api, kzgo)| {
					let stages = kzgo.as_ref().map(|map| map.bonuses).unwrap_or(0);
					(0..=stages).map(|stage| (global_api.id, stage, global_api.difficulty))
				})
				.collect::<Vec<_>>();

			trace!("fetched maps");

			let mut query = QueryBuilder::new(
				r#"
				INSERT INTO maps
					(id, name, global, workshop_id, filesize, approved_by, created_on, updated_on)
				"#,
			);

			let mut transaction = pool.begin().await.context("Failed to start transaction.")?;

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
					kzgo_map,
				)| {
					let filesize = (filesize > 0).then_some(filesize as i64);
					let approved_by =
						approved_by.map(|approved_by| approved_by.community_id() as i64);

					let workshop_id = 'scope: {
						if let Some(workshop_id) = kzgo_map.map(|map| map.workshop_id) {
							if let Ok(workshop_id) = workshop_id.parse::<i64>() {
								break 'scope Some(workshop_id);
							}
						}

						None
					};

					query
						.push_bind(id as i16)
						.push_bind(name)
						.push_bind(validated)
						.push_bind(workshop_id)
						.push_bind(filesize)
						.push_bind(approved_by)
						.push_bind(created_on)
						.push_bind(updated_on);
				},
			);

			query
				.build()
				.execute(&mut transaction)
				.await
				.context("Failed to insert maps into database.")?;

			transaction.commit().await.context("Failed to commit transaction.")?;

			let mut query = QueryBuilder::new(
				r#"
				INSERT INTO courses
					(id, map_id, stage, tier)
				"#,
			);

			let mut transaction = pool.begin().await.context("Failed to start transaction.")?;

			query.push_values(courses, |mut query, (map_id, stage, tier)| {
				trace!(?map_id, ?stage, ?tier, "inserting course");
				let course_id = map_id as i32 * 100 + stage as i32;

				let tier = (stage == 0).then_some(tier as i16);

				query
					.push_bind(course_id)
					.push_bind(map_id as i16)
					.push_bind(stage as i16)
					.push_bind(tier);
			});

			query
				.build()
				.execute(&mut transaction)
				.await
				.context("Failed to insert courses into database.")?;

			transaction.commit().await.context("Failed to commit transaction.")?;
		}

		Data::Servers => {
			let params = global_api::servers::Params {
				limit: Some(9999),
				..Default::default()
			};

			let servers = global_api::servers::root(&params, &gokz_client)
				.await
				.context("Failed to fetch servers")?;

			trace!(amount = %servers.len(), "fetched servers");

			for server in &servers {
				if sqlx::query!(
					"SELECT * FROM players WHERE id = $1",
					server.owner_id.community_id() as i64
				)
				.fetch_one(&pool)
				.await
				.is_err()
				{
					error!(?server, "server has unknown owner");

					let player = global_api::get_player(server.owner_id, &gokz_client).await?;

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
					.execute(&pool)
					.await?;

					info!(?player, "inserted missing player");
				}
			}

			let mut query = QueryBuilder::new(
				r#"
				INSERT INTO servers
					(id, name, owned_by)
				"#,
			);

			let mut transaction = pool.begin().await.context("Failed to start transaction.")?;

			query.push_values(
				servers,
				|mut query,
				 global_api::Server {
				     id,
				     name,
				     owner_id,
				     ..
				 }| {
					query
						.push_bind(id as i16)
						.push_bind(name)
						.push_bind(owner_id.community_id() as i64);
				},
			);

			query
				.build()
				.execute(&mut transaction)
				.await
				.context("Failed to insert servers into database.")?;

			transaction.commit().await.context("Failed to commit transaction.")?;
		}

		Data::Records {
			mut start_id,
		} => {
			const SLEEP_TIME: Duration = Duration::from_millis(727);
			loop {
				tokio::time::sleep(SLEEP_TIME).await;

				let global_api::Record {
					id,
					map_id,
					map_name,
					steam_id,
					player_name,
					server_id,
					stage,
					mode,
					teleports,
					time,
					created_on,
					..
				} = match global_api::get_record(start_id, &gokz_client).await {
					Ok(record) => record,
					Err(err) => match err {
						GokzError::Http {
							code, ..
						} if matches!(code.as_u16(), 500..=502) => {
							warn!(id = %start_id, "No records. Sleeping...");
							tokio::time::sleep(SLEEP_TIME).await;
							continue;
						}
						err => yeet!("GlobalAPI request failed: {err:?}"),
					},
				};

				let course_id = map_id as i32 * 100 + stage as i32;

				// Insert potentially new map
				if sqlx::query!("SELECT * FROM maps WHERE id = $1", map_id as i16)
					.fetch_one(&pool)
					.await
					.is_err()
				{
					let global_api_map = global_api::get_global_maps(9999, &gokz_client)
						.await
						.context("Failed to fetch maps from the GlobalAPI")?
						.into_iter()
						.find(|map| map.id == map_id)
						.ok_or(eyre!("Record with non global map? {map_name}"))?;

					let kzgo_map = kzgo_api::get_maps(&gokz_client)
						.await
						.context("Failed to fetch maps from KZ:GO")?
						.into_iter()
						.find(|map| map.id == map_id)
						.ok_or(eyre!("Record with non global map? {map_name}"))?;

					let courses = (0..=kzgo_map.bonuses)
						.map(|stage| {
							let course_id = global_api_map.id as i32 * 100 + stage as i32;
							(course_id, map_id as i16, stage, global_api_map.difficulty)
						})
						.collect::<Vec<_>>();

					let Ok(workshop_id) = kzgo_map.workshop_id.parse::<i64>() else {
						error!(?kzgo_map.workshop_id, "Invalid workshop id");
						start_id += 1;
						continue;
					};

					let approved_by = global_api_map
						.approved_by
						.map(|approved_by| approved_by.community_id() as i64);

					if let Some(steam_id) = approved_by {
						// Check if the player exists, or update info
						ensure_player(steam_id, None, &pool).await?;
					}

					// Insert map
					sqlx::query! {
						r#"
						INSERT INTO maps
							(
								id,
								name,
								global,
								workshop_id,
								filesize,
								created_on,
								updated_on
							)
						VALUES
							($1, $2, $3, $4, $5, $6, $7)
						"#,
						global_api_map.id as i16,
						global_api_map.name,
						global_api_map.validated,
						workshop_id,
						global_api_map.filesize as i64,
						DateTime::<Utc>::from_utc(
							NaiveDateTime::from_timestamp_opt(
								global_api_map.created_on.timestamp(),
								0
							)
							.unwrap(),
							Utc
						),
						DateTime::<Utc>::from_utc(
							NaiveDateTime::from_timestamp_opt(
								global_api_map.updated_on.timestamp(),
								0
							)
							.unwrap(),
							Utc
						),
					}
					.execute(&pool)
					.await
					.context("Failed to insert map into database.")?;

					let mut query = QueryBuilder::new(
						r#"
						INSERT INTO courses
							(id, map_id, stage, tier)
						"#,
					);

					// Insert courses
					query.push_values(courses, |mut query, (course_id, map_id, stage, tier)| {
						query
							.push_bind(course_id)
							.push_bind(map_id)
							.push_bind(stage as i16)
							.push_bind(tier as i16);
					});

					query
						.build()
						.execute(&pool)
						.await
						.context("Failed to insert courses into database.")?;
				}

				// Insert potentially new server
				if sqlx::query!("SELECT * FROM servers WHERE id = $1", server_id as i16,)
					.fetch_one(&pool)
					.await
					.is_err()
				{
					let server = global_api::get_server(server_id, &gokz_client)
						.await
						.context("Failed to fetch server")?;

					let owner_id = server.owner_id.community_id() as i64;

					// Check if the player exists, or update info
					ensure_player(owner_id, None, &pool).await?;

					sqlx::query! {
						r#"
						INSERT INTO servers
							(id, name, owned_by)
						VALUES
							($1, $2, $3)
						"#,
						server.id as i16,
						server.name,
						owner_id,
					}
					.execute(&pool)
					.await
					.context("Failed to insert server into database")?;
				}

				// Check if the player exists, or update info
				ensure_player(steam_id.community_id() as i64, Some(player_name.as_str()), &pool)
					.await?;

				// Insert record
				sqlx::query! {
					r#"
					INSERT INTO records
						(id, course_id, mode_id, player_id, server_id, time, teleports, created_on)
					VALUES
						($1, $2, $3, $4, $5, $6, $7, $8)
					"#,
					id as i32,
					course_id,
					mode as i16,
					steam_id.community_id() as i64,
					server_id as i16,
					time,
					teleports as i32,
					DateTime::<Utc>::from_utc(
						NaiveDateTime::from_timestamp_opt(created_on.timestamp(), 0).unwrap(),
						Utc
					),
				}
				.execute(&pool)
				.await
				.context("Failed to insert record into database.")?;

				info!(id = %start_id, "Inserted record.");

				start_id += 1;
			}
		}

		Data::Mappers => {
			let mappers = kzgo_api::get_maps(&gokz_client)
				.await?
				.into_iter()
				.flat_map(|map| map.mapper_ids.into_iter().map(move |steam_id| (steam_id, map.id)))
				.collect::<HashSet<_>>();

			for (steam_id, map_id) in &mappers {
				if sqlx::query!(
					"SELECT * FROM players WHERE id = $1",
					steam_id.community_id() as i64
				)
				.fetch_one(&pool)
				.await
				.is_err()
				{
					error!(?map_id, "map has unknown mapper");

					let player = global_api::get_player(*steam_id, &gokz_client).await?;

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
					.execute(&pool)
					.await?;

					info!(?player, "inserted missing mapper");
				}
			}

			let mut query = QueryBuilder::new("INSERT INTO mappers (player_id, map_id)");

			query.push_values(mappers, |mut query, (steam_id, map_id)| {
				query.push_bind(steam_id.community_id() as i64).push_bind(map_id as i16);
			});

			query.build().execute(&pool).await?;
		}

		Data::Filters => {
			let mut filters = Vec::new();

			for map in kzgo_api::get_maps(&gokz_client).await? {
				let course_id = map.id as i32 * 100;

				if !map.name.starts_with("skz_") && !map.name.starts_with("vnl_") {
					filters.push((course_id, 200_i16));
				}

				if map.skz {
					filters.push((course_id, 201_i16));
				}

				if map.vnl {
					filters.push((course_id, 202_i16));
				}
			}

			let mut query = QueryBuilder::new("INSERT INTO Filters (course_id, mode_id)");

			query.push_values(filters, |mut query, (course_id, mode_id)| {
				query.push_bind(course_id).push_bind(mode_id);
			});

			query.build().execute(&pool).await.context("Failed to insert filters.")?;
		}
	};

	info!(took = ?start.elapsed(), "Done.");

	Ok(())
}

async fn ensure_player(steam_id: i64, player_name: Option<&str>, pool: &PgPool) -> Result<()> {
	match sqlx::query!("SELECT * FROM players WHERE id = $1", steam_id).fetch_one(pool).await {
		Ok(player) => {
			// If the player just submitted a record, they cannot be banned.
			// Update the name, just in case they changed it.
			let mut query = QueryBuilder::new("UPDATE players SET is_banned = FALSE");

			if let Some(name) = player_name {
				query.push(", name = ").push_bind(name);
			}

			query.push(" WHERE id = ").push_bind(player.id);

			query.build().execute(pool).await.context("Failed to unban player")?;
		}
		Err(_) => {
			let player_name = player_name.unwrap_or("unknown");

			sqlx::query! {
				r#"
				INSERT INTO players
					(id, name, is_banned)
				VALUES
					($1, $2, $3)
				"#,
				steam_id,
				player_name,
				false,
			}
			.execute(pool)
			.await
			.context("Failed to insert player into database.")?;
		}
	}

	Ok(())
}
