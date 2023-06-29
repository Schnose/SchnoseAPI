use {
	crate::{
		database::modes::{Mode, ModeRow},
		models::app_state::AppState,
		Error, Result,
	},
	axum::{
		extract::{Path, State},
		http, Json,
	},
	std::sync::Arc,
};

#[utoipa::path(
	get,
	tag = "Modes",
	path = "/modes/{ident}",
	responses(
		(status = 200, body = Player),
		(status = 204, body = ()),
		(status = 500, body = Error),
	),
	params(
		("ident" = String, Path, description = "The mode's name or ID."),
	),
)]
#[tracing::instrument(level = "DEBUG", skip(state), err(Debug))]
pub async fn ident(
	method: http::Method,
	Path(mode): Path<gokz_rs::types::Mode>,
	State(state): State<Arc<AppState>>,
) -> Result<Json<Mode>> {
	let mode = sqlx::query_as::<_, ModeRow>("SELECT * FROM modes WHERE id = $1")
		.bind(mode as i16)
		.fetch_optional(state.db())
		.await?
		.map(TryInto::try_into)
		.ok_or(Error::NoContent)??;

	Ok(Json(mode))
}
