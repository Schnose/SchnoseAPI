pub mod ident;
pub use ident::ident;
use {
	crate::{
		database::modes::{Mode, ModeRow},
		models::app_state::AppState,
		Result,
	},
	axum::{extract::State, http, Json},
	std::sync::Arc,
};

#[utoipa::path(
	get,
	tag = "Modes",
	path = "/modes",
	responses(
		(status = 200, body = Vec<Mode>),
		(status = 500, body = Error),
	),
)]
#[tracing::instrument(level = "DEBUG", skip(state), err(Debug))]
pub async fn root(
	method: http::Method,
	State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Mode>>> {
	let modes = sqlx::query_as::<_, ModeRow>("SELECT * FROM modes")
		.fetch_all(state.db())
		.await?
		.into_iter()
		.map(TryInto::try_into)
		.collect::<Result<Vec<_>>>()?;

	assert_eq!(modes.len(), 3, "There should be 3 modes in the database.");

	Ok(Json(modes))
}