use {
	crate::{
		database::players::{Player, PlayerRow},
		models::app_state::AppState,
		Error, Result,
	},
	axum::{
		extract::{Path, State},
		http, Json,
	},
	color_eyre::eyre::Context,
	gokz_rs::types::PlayerIdentifier,
	sqlx::QueryBuilder,
	std::sync::Arc,
};

#[utoipa::path(
	get,
	tag = "Players",
	path = "/players/{ident}",
	responses(
		(status = 200, body = Player),
		(status = 204),
		(status = 400, description = "An invalid player identifier was provided."),
		(status = 500, body = Error),
	),
	params(
		("ident" = String, Path, description = "The player's name or SteamID."),
	),
)]
#[tracing::instrument(level = "DEBUG", skip(state), err(Debug))]
pub async fn ident(
	method: http::Method,
	Path(player_identifier): Path<PlayerIdentifier>,
	State(state): State<Arc<AppState>>,
) -> Result<Json<Player>> {
	let mut query = QueryBuilder::new("SELECT * FROM players WHERE ");

	match player_identifier {
		PlayerIdentifier::SteamID(steam_id) => {
			query.push("id = ").push_bind(steam_id.community_id() as i64);
		}
		PlayerIdentifier::Name(name) => {
			query.push("name ILIKE ").push_bind(format!("%{name}%"));
		}
	};

	query.push(" LIMIT 1 ");

	let player = query
		.build_query_as::<PlayerRow>()
		.fetch_optional(state.db())
		.await
		.context("Failed to fetch players from database.")?
		.ok_or(Error::NoContent)?
		.try_into()
		.context("Found invalid player in database.")?;

	Ok(Json(player))
}
