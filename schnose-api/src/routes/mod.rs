pub mod health;

use {
	crate::{models::app_state::AppState, SchnoseAPI},
	axum::{routing::get, Router},
	std::sync::Arc,
};

#[tracing::instrument(level = "TRACE", skip(app_state))]
pub fn router(app_state: Arc<AppState>) -> Router {
	Router::new()
		.merge(SchnoseAPI::swagger())
		.route("/health", get(health::root))
		.with_state(app_state)
}
