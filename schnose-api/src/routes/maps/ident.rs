use {
	crate::{response::Response, state::APIState},
	axum::extract::{Path, State},
	gokz_rs::{MapIdentifier, SteamID},
	schnose_api::{
		error::{yeet, Error},
		models::{Course, Map, Mapper},
	},
	schnosedb::models::{CourseRow, JoinedMapperRow, MapRow},
	tracing::{debug, trace},
};

#[axum::debug_handler]
pub async fn get(Path(map): Path<MapIdentifier>, State(state): State<APIState>) -> Response<Map> {
	trace!("GET /api/maps/{map:?}");

	let query = match map {
		MapIdentifier::ID(map_id) => {
			sqlx::query_as("SELECT * FROM maps WHERE id = ? LIMIT 1").bind(map_id)
		}
		MapIdentifier::Name(map_name) => {
			sqlx::query_as("SELECT * FROM maps WHERE name LIKE ? LIMIT 1")
				.bind(format!("%{map_name}%"))
		}
	};

	let map: MapRow = query
		.fetch_optional(state.db())
		.await?
		.ok_or(Error::NoContent)?;

	debug!("Map:\n\t{map:?}");

	let courses: Vec<CourseRow> = sqlx::query_as("SELECT * FROM courses WHERE map_id = ?")
		.bind(map.id)
		.fetch_all(state.db())
		.await?;

	debug!("Courses:\n\t{courses:?}");

	if courses.is_empty() {
		yeet!(Error::NoContent);
	}

	let courses = courses
		.into_iter()
		.filter_map(|row| {
			Some(Course {
				id: row.id,
				stage: row.stage,
				tier: row.tier.try_into().ok()?,
			})
		})
		.collect();

	let mappers: Vec<JoinedMapperRow> = sqlx::query_as(
		r#"
		SELECT
		  m.map_id,
		  m.mapper_id,
		  p.name AS mapper_name
		FROM mappers AS m
		JOIN players AS p ON p.id = m.mapper_id
		WHERE m.map_id = ?
		"#,
	)
	.bind(map.id)
	.fetch_all(state.db())
	.await?;

	debug!("Mappers:\n\t{mappers:?}");

	if mappers.is_empty() {
		yeet!(Error::NoContent);
	}

	let mappers = mappers
		.into_iter()
		.filter_map(|row| {
			if row.mapper_id == 0 {
				return None;
			}

			Some(Mapper {
				name: row.mapper_name,
				steam_id: SteamID::from_id32(row.mapper_id),
			})
		})
		.collect();

	Ok(Map {
		id: map.id,
		name: map.name,
		global: map.global,
		filesize: map.filesize,
		courses,
		mappers,
		approved_by: (map.approved_by == 0).then_some(SteamID::from_id32(map.approved_by)),
		created_on: map.created_on,
		updated_on: map.updated_on,
	}
	.into())
}
