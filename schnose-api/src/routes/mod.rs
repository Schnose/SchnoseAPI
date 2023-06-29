pub mod health;
pub mod maps;
pub mod modes;
pub mod players;

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
		.route("/players", get(players::root))
		.route("/players/:ident", get(players::ident))
		.route("/modes", get(modes::root))
		.route("/modes/:ident", get(modes::ident))
		.route("/maps", get(maps::root))
		.route("/maps/:ident", get(maps::ident))
		.with_state(app_state)
}
