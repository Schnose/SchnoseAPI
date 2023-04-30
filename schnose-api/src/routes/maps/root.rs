use {
	crate::{response::Response, state::APIState},
	axum::extract::{Query, State},
	gokz_rs::SteamID,
	itertools::Itertools,
	schnose_api::{
		error::{yeet, Error},
		models::{Map, MapQuery, Mapper},
	},
	serde::Deserialize,
	sqlx::QueryBuilder,
	tracing::{debug, trace},
};

#[derive(Debug, Deserialize)]
pub struct Params {
	name: Option<String>,
	global: Option<bool>,
	limit: Option<u16>,
}

#[axum::debug_handler]
pub async fn get(
	Query(params): Query<Params>,
	State(state): State<APIState>,
) -> Response<Vec<Map>> {
	trace!("GET /api/maps");
	trace!("{params:?}");

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

	let mut filter = " WHERE ";

	if let Some(name) = params.name {
		query
			.push(" WHERE map.name LIKE ")
			.push_bind(format!("%{name}%"));
		filter = " AND ";
	}

	if let Some(global) = params.global {
		query
			.push(filter)
			.push(" map.global = ")
			.push_bind(global as u8);
		filter = " AND ";
	}

	query
		.push(" GROUP BY map.id ")
		.push(" LIMIT ")
		.push_bind(match params.limit.unwrap_or(1200) {
			0 => 1,
			limit @ (1..=1200) => limit,
			1201.. => 1200,
		});

	let maps: Vec<MapQuery> = query
		.build_query_as()
		.fetch_all(state.db())
		.await?;

	debug!("Maps:\n\t{maps:?}");

	if maps.is_empty() {
		yeet!(Error::NoContent);
	}

	let maps = maps
		.into_iter()
		.map(|map| {
			let mappers = map
				.mappers
				.0
				.into_iter()
				.sorted_by(|a, b| a.steam_id.cmp(&b.steam_id))
				.dedup_by(|a, b| a.steam_id == b.steam_id)
				.filter_map(|mapper| {
					(mapper.steam_id != 0).then_some(Mapper {
						name: mapper.name,
						steam_id: SteamID::from_id32(mapper.steam_id),
					})
				})
				.collect();
			Map {
				id: map.id,
				name: map.name,
				global: map.global,
				filesize: map.filesize,
				courses: map
					.courses
					.0
					.into_iter()
					.sorted_by(|a, b| a.id.cmp(&b.id))
					.dedup_by(|a, b| a.id == b.id)
					.collect(),
				mappers,
				approved_by: map
					.approved_by
					.and_then(|id| (id == 0).then_some(SteamID::from_id32(id))),
				created_on: map.created_on,
				updated_on: map.updated_on,
			}
		})
		.collect_vec();

	Ok(maps.into())
}
