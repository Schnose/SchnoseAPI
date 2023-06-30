use {
	crate::{
		database::{Record, RecordRow},
		models::app_state::AppState,
		Error, Result,
	},
	axum::{
		extract::{Path, State},
		http, Json,
	},
	color_eyre::eyre::Context,
	std::sync::Arc,
};

#[utoipa::path(
	get,
	tag = "Records",
	path = "/api/records/{id}",
	responses(
		(status = 200, body = Record),
		(status = 204),
		(status = 400, description = "An invalid record id was provided."),
		(status = 500, body = Error),
	),
	params(
		("id" = i32, Path, description = "The record's id."),
	),
)]
#[tracing::instrument(level = "DEBUG", skip(state), err(Debug))]
pub async fn id(
	method: http::Method,
	Path(id): Path<u32>,
	State(state): State<Arc<AppState>>,
) -> Result<Json<Record>> {
	let record = sqlx::query_as::<_, RecordRow>(
		r#"
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
	)
	.fetch_optional(state.db())
	.await
	.context("Failed to fetch record from database.")?
	.ok_or(Error::NoContent)?
	.try_into()
	.context("Found invalid record in database.")?;

	Ok(Json(record))
}
