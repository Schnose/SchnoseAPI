use {
	crate::{response::Response, state::APIState},
	axum::extract::{Path, State},
	gokz_rs::{MapIdentifier, SteamID},
	itertools::Itertools,
	schnose_api::{
		error::Error,
		models::{Course, Map, MapQuery, Mapper},
	},
	sqlx::QueryBuilder,
	tracing::{debug, trace},
};

#[axum::debug_handler]
pub async fn get(Path(map): Path<MapIdentifier>, State(state): State<APIState>) -> Response<Map> {
	trace!("GET /api/maps/{map:?}");

	let mut query = QueryBuilder::new(
		r#"
		SELECT
		  map.*,
		  JSON_ARRAYAGG(
		    JSON_OBJECT(
		      "id",    course.id,
		      "stage", course.stage,
		      "tier",  course.tier
		    )
		  ) as courses,
		  JSON_ARRAYAGG(
		    JSON_OBJECT(
		      "name",     player.name,
		      "steam_id", mapper.mapper_id
		    )
		  ) as mappers
		FROM maps AS map
		JOIN courses AS course ON course.map_id = map.id
		JOIN mappers AS mapper ON mapper.map_id = map.id
		JOIN players AS player ON player.id = mapper.mapper_id
		"#,
	);

	match map {
		MapIdentifier::ID(map_id) => {
			query
				.push(" WHERE map.id = ")
				.push_bind(map_id);
		}
		MapIdentifier::Name(map_name) => {
			query
				.push(" WHERE map.name LIKE ")
				.push_bind(format!("%{map_name}%"));
		}
	};

	query
		.push(" GROUP BY map.id ")
		.push(" LIMIT 1 ");

	let map: MapQuery = query
		.build_query_as()
		.fetch_optional(state.db())
		.await?
		.ok_or(Error::NoContent)?;

	debug!("Map:\n\t{map:?}");

	// TODO: Do this in the database instead of here. It's fine in this case since there aren't that
	// many maps anyway, but I would still like to do this properly.
	let courses = map
		.courses
		.0
		.into_iter()
		.flat_map(Course::try_from)
		.sorted_by(|a, b| a.id.cmp(&b.id))
		.dedup_by(|a, b| a.id == b.id)
		.collect();

	let mappers = map
		.mappers
		.0
		.into_iter()
		.flat_map(Mapper::try_from)
		.sorted_by(|a, b| a.steam_id.cmp(&b.steam_id))
		.dedup_by(|a, b| a.steam_id == b.steam_id)
		.collect_vec();

	Ok(Map {
		id: map.id,
		name: map.name,
		global: map.global,
		filesize: map.filesize,
		courses,
		mappers,
		approved_by: map
			.approved_by
			.and_then(|id| (id == 0).then_some(SteamID::from_id32(id))),
		created_on: map.created_on,
		updated_on: map.updated_on,
	}
	.into())
}
