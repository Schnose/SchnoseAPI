pub mod health;

use {
	crate::{models::app_state::AppState, SchnoseAPI},
	axum::{routing::get, Router},
	std::sync::Arc,
};

pub fn router(app_state: Arc<AppState>) -> Router {
	Router::new()
		.merge(SchnoseAPI::swagger())
		.route("/health", get(health::root))
		.with_state(app_state)
}
