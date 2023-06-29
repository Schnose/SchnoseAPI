use {
	crate::{
		database::maps::{MapModel, MapRow},
		models::app_state::AppState,
		Error, Result,
	},
	axum::{
		extract::{Path, State},
		http, Json,
	},
	color_eyre::eyre::Context,
	gokz_rs::types::MapIdentifier,
	sqlx::QueryBuilder,
	std::sync::Arc,
};

#[utoipa::path(
	get,
	tag = "Maps",
	path = "/maps/{ident}",
	responses(
		(status = 200, body = Player),
		(status = 400, description = "An invalid map was provided."),
		(status = 500, body = Error),
	),
	params(
		("ident" = String, Path, description = "The map's name or ID."),
	),
)]
#[tracing::instrument(level = "DEBUG", skip(state), err(Debug))]
pub async fn ident(
	method: http::Method,
	Path(map_identifier): Path<MapIdentifier>,
	State(state): State<Arc<AppState>>,
) -> Result<Json<MapModel>> {
	let mut query = QueryBuilder::new(
		r#"
		SELECT
			map.*,
			JSON_AGG(DISTINCT course.*) courses,
			JSON_AGG(DISTINCT p_mapper.*) mappers
			FROM maps map
		JOIN courses course
			ON course.map_id = map.id
		JOIN mappers mapper
			ON mapper.map_id = map.id
		JOIN players p_mapper
			ON p_mapper.id = mapper.player_id
		WHERE
		"#,
	);

	match map_identifier {
		MapIdentifier::Id(map_id) => query.push(" map.id = ").push_bind(map_id as i16),
		MapIdentifier::Name(map_name) => {
			query.push(" map.name ILIKE ").push_bind(format!("%{map_name}%"))
		}
	};

	query.push(" GROUP BY map.id ");

	let map = query
		.build_query_as::<MapRow>()
		.fetch_optional(state.db())
		.await
		.context("Failed to fetch map from database.")?
		.ok_or(Error::NoContent)?
		.try_into()
		.context("Found invalid map in database.")?;

	Ok(Json(map))
}
