pub mod ident;
pub use ident::ident;
use {
	crate::{
		database::{MapModel, MapRow},
		models::app_state::AppState,
		Error, Filter, Result,
	},
	axum::{
		extract::{Query, State},
		http, Json,
	},
	chrono::{DateTime, Utc},
	color_eyre::eyre::Context,
	gokz_rs::types::PlayerIdentifier,
	serde::Deserialize,
	sqlx::QueryBuilder,
	std::sync::Arc,
	utoipa::IntoParams,
};

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct Params {
	pub name: Option<String>,
	#[param(value_type = Option<String>)]
	pub mapper: Option<PlayerIdentifier>,
	pub global: Option<bool>,
	pub created_after: Option<DateTime<Utc>>,
	pub created_before: Option<DateTime<Utc>>,
	pub offset: Option<i64>,
	pub limit: Option<u64>,
}

#[utoipa::path(
	get,
	tag = "Maps",
	path = "/api/maps",
	responses(
		(status = 200, body = Vec<MapModel>),
		(status = 204),
		(status = 400, body = Error),
		(status = 500, body = Error),
	),
	params(Params),
)]
#[tracing::instrument(level = "DEBUG", skip(state), err(Debug))]
pub async fn root(
	method: http::Method,
	Query(Params {
		name,
		mapper,
		global,
		created_after,
		created_before,
		offset,
		limit,
	}): Query<Params>,
	State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<MapModel>>> {
	let mut query = QueryBuilder::new(
		r#"
		SELECT
			map.*,
			JSON_AGG(course.* ORDER BY course.id) courses,
			JSON_AGG(p_mapper.* ORDER BY p_mapper.name) mappers
		FROM maps map
		JOIN courses course
			ON course.map_id = map.id
		JOIN mappers mapper
			ON mapper.map_id = map.id
		JOIN players p_mapper
			ON p_mapper.id = mapper.player_id
		"#,
	);

	let mut filter = Filter::Where;

	if let Some(name) = name {
		query.push(filter).push("map.name ILIKE ").push_bind(format!("%{name}%"));
		filter.and();
	}

	if let Some(mapper) = mapper {
		query.push(filter);

		match mapper {
			PlayerIdentifier::SteamID(steam_id) => {
				query.push("mapper.player_id = ").push_bind(steam_id.community_id() as i64);
			}

			PlayerIdentifier::Name(name) => {
				query
					.push("mapper.player_id = (SELECT id FROM players p WHERE p.name ILIKE ")
					.push_bind(format!("%{name}%"))
					.push(" LIMIT 1)");
			}
		};

		filter.and();
	}

	if let Some(global) = global {
		query.push(filter).push("map.global = ").push_bind(global);
		filter.and();
	}

	match (created_after, created_before) {
		(None, None) => {}

		(None, Some(created_before)) => {
			query.push(filter).push(" map.created_on < ").push_bind(created_before);
		}

		(Some(created_after), None) => {
			query.push(filter).push(" map.created_on > ").push_bind(created_after);
		}

		(Some(created_after), Some(created_before)) => {
			if created_after > created_before {
				return Err(Error::InvalidDates);
			}

			query
				.push(filter)
				.push(" map.created_on < ")
				.push_bind(created_before)
				.push(" AND map.created_on > ")
				.push_bind(created_after);
		}
	};

	query
		.push(" GROUP BY map.id ")
		.push(" LIMIT ")
		.push_bind(limit.map_or(1000, |limit| (limit as i64).min(2000)))
		.push(" OFFSET ")
		.push_bind(offset.unwrap_or(0));

	let maps = query
		.build_query_as::<MapRow>()
		.fetch_all(state.db())
		.await
		.context("Failed to fetch maps from database.")?
		.into_iter()
		.map(TryInto::try_into)
		.collect::<Result<Vec<_>>>()
		.context("Found invalid map in database.")?;

	if maps.is_empty() {
		return Err(Error::NoContent);
	}

	Ok(Json(maps))
}
