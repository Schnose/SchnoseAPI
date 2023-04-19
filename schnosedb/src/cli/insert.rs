use {
	color_eyre::{
		eyre::{bail as yeet, Context},
		Result,
	},
	schnosedb::models::*,
	sqlx::{query_builder::Separated, MySql, Pool, QueryBuilder},
	tracing::info,
};

#[tracing::instrument(skip(values, push_values, database_connection))]
pub async fn insert_rows<'args, Values, F>(
	table: &str,
	database_connection: &Pool<MySql>,
	values_query_part: &str,
	values: Values,
	push_values: F,
) -> Result<()>
where
	Values: IntoIterator,
	F: FnMut(Separated<'_, 'args, MySql, &'static str>, Values::Item),
{
	let mut query = QueryBuilder::new(format!("INSERT INTO {table} ({values_query_part}) "));
	query.push_values(values, push_values);

	query
		.build()
		.execute(database_connection)
		.await
		.context("Failed to insert rows into database.")?;

	Ok(())
}

pub async fn insert_rows_from_json(
	json: String,
	table: &str,
	database_connection: &Pool<MySql>,
) -> Result<()> {
	match table {
		"modes" => {
			let modes: Vec<ModeRow> =
				serde_json::from_str(&json).context("Failed to deserialize rows.")?;

			insert_rows(table, database_connection, "id, name", &modes, |mut query, mode| {
				query
					// This results in
					// +-----+------------+
					// | id  | name       |
					// +-----+------------+
					// | -56 | kz_timer   |
					// | -55 | kz_simple  |
					// | -54 | kz_vanilla |
					// +-----+------------+
					// if I don't cast it as at least `u16`
					// :thonk:
					.push_bind(mode.id as u16)
					.push_bind(&mode.name);
			})
			.await?;

			info!("Inserted {total} rows into `{table}`.", total = modes.len());
		}
		"players" => {
			let players: Vec<PlayerRow> =
				serde_json::from_str(&json).context("Failed to deserialize rows.")?;

			insert_rows(
				table,
				database_connection,
				"id, name, is_banned",
				&players,
				|mut query, player| {
					query
						.push_bind(player.id)
						.push_bind(&player.name)
						.push_bind(player.is_banned as u8);
				},
			)
			.await?;

			info!("Inserted {total} rows into `{table}`.", total = players.len());
		}
		"maps" => {
			let maps: Vec<MapRow> =
				serde_json::from_str(&json).context("Failed to deserialize rows.")?;

			insert_rows(
				table,
				database_connection,
				"id, name, global, filesize, approved_by, created_on, updated_on",
				&maps,
				|mut query, map| {
					query
						.push_bind(map.id)
						.push_bind(&map.name)
						.push_bind(map.global as u8)
						.push_bind(map.filesize)
						.push_bind(map.approved_by)
						.push_bind(map.created_on)
						.push_bind(map.updated_on);
				},
			)
			.await?;

			info!("Inserted {total} rows into `{table}`.", total = maps.len());
		}
		"mappers" => {
			let mappers: Vec<MapperRow> =
				serde_json::from_str(&json).context("Failed to deserialize rows.")?;

			insert_rows(
				table,
				database_connection,
				"map_id, mapper_id",
				&mappers,
				|mut query, mapper| {
					query
						.push_bind(mapper.map_id)
						.push_bind(mapper.mapper_id);
				},
			)
			.await?;

			info!("Inserted {total} rows into `{table}`.", total = mappers.len());
		}
		"courses" => {
			let courses: Vec<CourseRow> =
				serde_json::from_str(&json).context("Failed to deserialize rows.")?;

			insert_rows(
				table,
				database_connection,
				"id, map_id, stage, tier",
				&courses,
				|mut query, course| {
					query
						.push_bind(course.id)
						.push_bind(course.map_id)
						.push_bind(course.stage)
						.push_bind(course.tier as u16);
				},
			)
			.await?;

			info!("Inserted {total} rows into `{table}`.", total = courses.len());
		}
		"filters" => {
			let filters: Vec<FilterRow> =
				serde_json::from_str(&json).context("Failed to deserialize rows.")?;

			insert_rows(
				table,
				database_connection,
				"course_id, mode_id",
				&filters,
				|mut query, filter| {
					query
						.push_bind(filter.course_id)
						.push_bind(filter.mode_id as u16);
				},
			)
			.await?;

			info!("Inserted {total} rows into `{table}`.", total = filters.len());
		}
		"servers" => {
			let servers: Vec<ServerRow> =
				serde_json::from_str(&json).context("Failed to deserialize rows.")?;

			insert_rows(
				table,
				database_connection,
				"id, name, owned_by, approved_by",
				&servers,
				|mut query, server| {
					query
						.push_bind(server.id)
						.push_bind(&server.name)
						.push_bind(server.owned_by)
						.push_bind(server.approved_by);
				},
			)
			.await?;

			info!("Inserted {total} rows into `{table}`.", total = servers.len());
		}
		"records" => {
			let records: Vec<RecordRow> =
				serde_json::from_str(&json).context("Failed to deserialize rows.")?;

			insert_rows(
				table,
				database_connection,
				"id, course_id, mode_id, player_id, server_id, time, teleports, created_on",
				&records,
				|mut query, record| {
					query
						.push_bind(record.id)
						.push_bind(record.course_id)
						.push_bind(record.mode_id as u16)
						.push_bind(record.player_id)
						.push_bind(record.server_id)
						.push_bind(record.time)
						.push_bind(record.teleports)
						.push_bind(record.created_on);
				},
			)
			.await?;

			info!("Inserted {total} rows into `{table}`.", total = records.len());
		}
		invalid_table => yeet!("`{invalid_table}` is not a valid table."),
	};

	Ok(())
}
