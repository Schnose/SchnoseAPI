use {
	crate::{response::Response, state::APIState},
	axum::extract::{Path, State},
	schnose_api::{error::Error, models::Mode},
	schnosedb::models::ModeRow,
	tracing::{debug, trace},
};

#[axum::debug_handler]
pub async fn get(Path(mode): Path<gokz_rs::Mode>, State(state): State<APIState>) -> Response<Mode> {
	trace!("GET /api/modes/{:?}", mode.api());

	let mode: ModeRow = sqlx::query_as("SELECT * FROM modes WHERE id = ?")
		.bind(mode as u16)
		.fetch_optional(state.db())
		.await?
		.ok_or(Error::NoContent)?;

	debug!("Mode:\n\t{mode:?}");

	Ok(Mode::from(mode).into())
}
