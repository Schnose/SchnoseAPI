mod args;
mod config;

use {
	crate::{args::Args, config::Config},
	args::Data,
	chrono::NaiveDateTime,
	clap::Parser,
	color_eyre::{
		eyre::{bail as yeet, eyre, Context},
		Result,
	},
	gokz_rs::{global_api, kzgo_api, prelude::Error as GokzError},
	sqlx::{postgres::PgPoolOptions, QueryBuilder},
	std::{
		collections::HashMap,
		time::{Duration, Instant},
	},
	tracing::{info, trace, warn},
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

				trace!("fetched players");

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
							.push_bind(steam_id.community_id() as i32)
							.push_bind(name)
							.push_bind(is_banned);
					},
				);

				query
					.build()
					.execute(&mut transaction)
					.await
					.context("Failed to insert players into database.")?;

				transaction.commit().await.context("Failed to commit transaction.")?;

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
				.filter_map(|global_api_map| {
					let kzgo_map = kzgo_maps.remove(&global_api_map.name)?;
					Some((global_api_map, kzgo_map))
				})
				.collect::<Vec<_>>();

			let courses = maps
				.iter()
				.map(|(global_api, kzgo)| (global_api.id, kzgo.bonuses + 1, global_api.difficulty))
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
					kzgo_api::Map {
						workshop_id, ..
					},
				)| {
					let filesize = (filesize > 0).then_some(filesize as i64);
					let approved_by =
						approved_by.map(|approved_by| approved_by.community_id() as i32);

					query
						.push_bind(id as i16)
						.push_bind(name)
						.push_bind(validated)
						.push_bind(workshop_id as i32)
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

			trace!("fetched servers");

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
						.push_bind(owner_id.community_id() as i32);
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
						} if matches!(code, 500..=502) => {
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

					// Insert map
					sqlx::query! {
						r#"
						INSERT INTO maps
							(id, name, global, workshop_id, filesize, approved_by, created_on, updated_on)
						VALUES
							($1, $2, $3, $4, $5, $6, $7, $8)
						"#,
						global_api_map.id as i16,
						global_api_map.name,
						global_api_map.validated,
						kzgo_map.workshop_id as i32,
						global_api_map.filesize as i64,
						global_api_map
							.approved_by
							.map(|approved_by| approved_by.community_id() as i32),
						NaiveDateTime::from_timestamp_opt(global_api_map.created_on.timestamp(), 0)
							.unwrap(),
						NaiveDateTime::from_timestamp_opt(global_api_map.updated_on.timestamp(), 0)
							.unwrap()
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

				// Check if the player exists, or update info
				match sqlx::query!(
					"SELECT * FROM players WHERE id = $1",
					steam_id.community_id() as i32
				)
				.fetch_one(&pool)
				.await
				{
					Ok(player) => {
						// If the player just submitted a record, they cannot be banned.
						// Update the name, just in case they changed it.
						sqlx::query! {
							r#"
						UPDATE players
						SET is_banned = FALSE, name = $2
						WHERE id = $1
						"#,
							player.id,
							player_name,
						}
						.execute(&pool)
						.await
						.context("Failed to unban player")?;
					}
					Err(_) => {
						sqlx::query! {
							r#"
						INSERT INTO players
							(id, name, is_banned)
						VALUES
							($1, $2, $3)
						"#,
							steam_id.community_id() as i32,
							player_name,
							false,
						}
						.execute(&pool)
						.await
						.context("Failed to insert player into database.")?;
					}
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

					sqlx::query! {
						r#"
					INSERT INTO servers
						(id, name, owned_by)
					VALUES
						($1, $2, $3)
					"#,
						server.id as i16,
						server.name,
						server.owner_id.community_id() as i32,
					}
					.execute(&pool)
					.await
					.context("Failed to insert server into database")?;
				}

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
					steam_id.community_id() as i32,
					server_id as i16,
					time,
					teleports as i32,
					NaiveDateTime::from_timestamp_opt(created_on.timestamp(), 0)
						.unwrap()
				}
				.execute(&pool)
				.await
				.context("Failed to insert record into database.")?;

				info!(id = %start_id, "Inserted record.");

				start_id += 1;
				tokio::time::sleep(SLEEP_TIME).await;
			}
		}
	};

	info!(took = ?start.elapsed(), "Done.");

	Ok(())
}
