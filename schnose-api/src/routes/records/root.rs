use {
	crate::{response::Response, state::APIState},
	axum::extract::State,
	schnose_api::{
		error::Error,
		models::{Record, RecordQuery},
	},
	tracing::{debug, trace},
};

// TODO
// #[derive(Debug, Deserialize)]
// pub struct Params {
// }

#[axum::debug_handler]
pub async fn get(
	// Query(params): Query<Params>,
	State(state): State<APIState>,
) -> Response<Record> {
	trace!("GET /api/records");
	// trace!("{params:?}");

	let record: RecordQuery = sqlx::query_as(
		r#"
		SELECT
		  record.id,
		  map.id AS map_id,
		  map.name AS map_name,
		  JSON_OBJECT(
		    "id",    _course.id,
		    "stage", _course.stage,
		    "tier",  _course.tier
		  ) AS course,
		  record.mode_id,
		  JSON_OBJECT(
		    "id",        player.id,
		    "name",      player.name,
		    "is_banned", player.is_banned
		  ) AS player,
		  JSON_OBJECT(
		    "id", server.id,
		    "name", server.name,
		    "owned_by", JSON_OBJECT(
		      "name", server_owner.name,
		      "steam_id", server_owner.id
		    ),
		    "approved_by", server.approved_by
		  ) AS server,
		  record.time,
		  record.teleports,
		  record.created_on
		FROM records AS record
		JOIN courses AS _course ON _course.id = record.course_id
		JOIN maps AS map ON map.id = _course.map_id
		JOIN players AS player ON player.id = record.player_id
		JOIN servers AS server ON server.id = record.server_id
		JOIN players AS server_owner ON server_owner.id = server.owned_by
		ORDER BY created_on DESC
		LIMIT 1
		"#,
	)
	.fetch_optional(state.db())
	.await?
	.ok_or(Error::NoContent)?;

	debug!("Record:\n\t{record:?}");

	Ok(Record::try_from(record)?.into())
}
