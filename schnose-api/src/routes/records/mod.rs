pub mod id;
pub use id::id;
use {
	crate::{
		database::{Record, RecordRow},
		models::app_state::AppState,
		Error, Filter, Result,
	},
	axum::{
		extract::{Query, State},
		http, Json,
	},
	chrono::{DateTime, Utc},
	color_eyre::eyre::Context,
	gokz_rs::types::{MapIdentifier, Mode, PlayerIdentifier, Runtype, ServerIdentifier},
	serde::Deserialize,
	sqlx::QueryBuilder,
	std::sync::Arc,
	utoipa::IntoParams,
};

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct Params {
	#[param(value_type = Option<String>)]
	pub player: Option<PlayerIdentifier>,
	#[param(value_type = Option<String>)]
	pub mode: Option<Mode>,
	#[param(value_type = Option<String>)]
	pub runtype: Option<Runtype>,
	#[param(value_type = Option<String>)]
	pub map: Option<MapIdentifier>,
	pub stage: Option<u8>,
	#[param(value_type = Option<String>)]
	pub server: Option<ServerIdentifier>,
	pub created_after: Option<DateTime<Utc>>,
	pub created_before: Option<DateTime<Utc>>,
	pub offset: Option<i64>,
	pub limit: Option<u64>,
}

#[utoipa::path(
	get,
	tag = "Records",
	path = "/api/records",
	responses(
		(status = 200, body = Vec<Record>),
		(status = 204),
		(status = 400, description = "Invalid parameters were provided."),
		(status = 500, body = Error),
	),
	params(Params)
)]
#[tracing::instrument(level = "DEBUG", skip(state), err(Debug))]
pub async fn root(
	method: http::Method,
	Query(Params {
		player,
		mode,
		runtype,
		map,
		stage,
		server,
		created_after,
		created_before,
		offset,
		limit,
	}): Query<Params>,
	State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Record>>> {
	let mut query = QueryBuilder::new(
		r#"
		SELECT * FROM (
			SELECT
				r.id,
				ROW_TO_JSON(c) course,
				m.name map_name,
				r.mode_id,
				r.player_id,
				p.name player_name,
				r.server_id,
				s.name server_name,
				r.time,
				r.teleports,
				r.created_on
			FROM records r
			JOIN courses c
				ON c.id = r.course_id
			JOIN maps m
				ON m.id = c.map_id
			JOIN players p
				ON p.id = r.player_id
			JOIN servers s
				ON s.id = r.server_id
		"#,
	);

	let mut filter = Filter::Where;

	if let Some(player_identifier) = player {
		query.push(filter);

		match player_identifier {
			PlayerIdentifier::SteamID(steam_id) => {
				query.push("p.id = ").push_bind(steam_id.community_id() as i64);
			}

			PlayerIdentifier::Name(name) => {
				let player_id =
					sqlx::query!("SELECT id FROM players WHERE name LIKE $1", format!("%{name}%"))
						.fetch_optional(state.db())
						.await
						.context("Failed to fetch player from database.")?
						.ok_or(Error::NoContent)?
						.id;

				query.push("p.id = ").push_bind(player_id);
			}
		};

		filter.and();
	}

	if let Some(mode) = mode {
		query.push(filter).push("r.mode_id = ").push_bind(mode as i16);

		filter.and();
	}

	if let Some(runtype) = runtype {
		query.push(filter).push("r.teleports ");

		match runtype {
			Runtype::TP => {
				query.push("> 0");
			}

			Runtype::Pro => {
				query.push("= 0");
			}
		};

		filter.and();
	}

	if let Some(map_identifier) = map {
		query.push(filter);

		match map_identifier {
			MapIdentifier::Id(map_id) => {
				query.push("m.id = ").push_bind(map_id as i16);
			}

			MapIdentifier::Name(name) => {
				let map_id =
					sqlx::query!("SELECT id FROM maps WHERE name LIKE $1", format!("%{name}%"))
						.fetch_optional(state.db())
						.await
						.context("Failed to fetch map from database.")?
						.ok_or(Error::NoContent)?
						.id;

				query.push("m.id = ").push_bind(map_id);
			}
		};

		filter.and();
	}

	if let Some(stage) = stage {
		query.push(filter).push("c.stage = ").push_bind(stage as i16);
		filter.and();
	}

	if let Some(server_identifier) = server {
		query.push(filter);

		match server_identifier {
			ServerIdentifier::Id(server_id) => {
				query.push("s.id = ").push_bind(server_id as i16);
			}

			ServerIdentifier::Name(name) => {
				let server_id =
					sqlx::query!("SELECT id FROM servers WHERE name LIKE $1", format!("%{name}%"))
						.fetch_optional(state.db())
						.await
						.context("Failed to fetch server from database.")?
						.ok_or(Error::NoContent)?
						.id;

				query.push("s.id = ").push_bind(server_id);
			}
		};
	}

	match (created_after, created_before) {
		(None, None) => {}

		(None, Some(created_before)) => {
			query.push(filter).push(" r.created_on < ").push_bind(created_before);
		}

		(Some(created_after), None) => {
			query.push(filter).push(" r.created_on > ").push_bind(created_after);
		}

		(Some(created_after), Some(created_before)) => {
			if created_after > created_before {
				return Err(Error::InvalidDates);
			}

			query
				.push(filter)
				.push(" r.created_on < ")
				.push_bind(created_before)
				.push(" AND r.created_on > ")
				.push_bind(created_after);
		}
	};

	query
		.push(
			r#"
				ORDER BY r.created_on DESC
			) r_outer
			"#,
		)
		.push(" LIMIT ")
		.push_bind(limit.map_or(100, |limit| (limit as i64).min(1000)))
		.push(" OFFSET ")
		.push_bind(offset.unwrap_or(0));

	let records = query
		.build_query_as::<RecordRow>()
		.fetch_all(state.db())
		.await
		.context("Failed to fetch records from database.")?
		.into_iter()
		.map(TryInto::try_into)
		.collect::<Result<Vec<Record>>>()
		.context("Found invalid record in database.")?;

	if records.is_empty() {
		return Err(Error::NoContent);
	}

	Ok(Json(records))
}
