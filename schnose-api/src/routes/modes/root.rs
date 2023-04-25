use {
	crate::{response::Response, state::APIState},
	axum::extract::State,
	itertools::Itertools,
	schnose_api::models::Mode,
	schnosedb::models::ModeRow,
	tracing::{debug, trace},
};

#[axum::debug_handler]
pub async fn get(State(state): State<APIState>) -> Response<Vec<Mode>> {
	trace!("GET /api/modes");

	let modes: Vec<ModeRow> = sqlx::query_as("SELECT * FROM modes")
		.fetch_all(state.db())
		.await?;

	debug!("Modes:\n\t{modes:?}");

	Ok(modes
		.into_iter()
		.map(Into::into)
		.inspect(|mode| debug!("Parsed mode: {mode:?}"))
		.collect_vec()
		.into())
}
