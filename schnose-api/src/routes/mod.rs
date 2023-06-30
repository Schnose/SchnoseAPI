pub mod filters;
pub mod health;
pub mod maps;
pub mod modes;
pub mod players;
pub mod records;
pub mod servers;

use {
	crate::{models::app_state::AppState, SchnoseAPI},
	axum::{routing::get, Router},
	std::sync::Arc,
};

#[tracing::instrument(level = "TRACE", skip(app_state))]
pub fn router(app_state: Arc<AppState>) -> Router {
	Router::new()
		.merge(SchnoseAPI::swagger())
		.route("/api/health", get(health::root))
		.route("/api/players", get(players::root))
		.route("/api/players/:ident", get(players::ident))
		.route("/api/modes", get(modes::root))
		.route("/api/modes/:ident", get(modes::ident))
		.route("/api/maps", get(maps::root))
		.route("/api/maps/:ident", get(maps::ident))
		.route("/api/servers", get(servers::root))
		.route("/api/servers/:ident", get(servers::ident))
		.route("/api/filters/map/:ident", get(filters::map))
		.route("/api/records", get(records::root))
		.route("/api/records/:id", get(records::id))
		.with_state(app_state)
}
