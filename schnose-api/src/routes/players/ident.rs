use {
	crate::{response::Response, state::APIState},
	axum::extract::{Path, State},
	gokz_rs::PlayerIdentifier,
	schnose_api::{error::Error, models::Player},
	schnosedb::models::PlayerRow,
	tracing::{debug, trace},
};

#[axum::debug_handler]
pub async fn get(
	Path(player): Path<PlayerIdentifier>,
	State(state): State<APIState>,
) -> Response<Player> {
	trace!("GET /api/players/{player:?}");

	let query = match player {
		PlayerIdentifier::SteamID(steam_id) => {
			sqlx::query_as("SELECT * FROM players WHERE id = ? LIMIT 1").bind(steam_id.as_id32())
		}
		PlayerIdentifier::Name(player_name) => {
			sqlx::query_as("SELECT * FROM players WHERE name LIKE ? LIMIT 1")
				.bind(format!("%{player_name}%"))
		}
	};

	let player: PlayerRow = query
		.fetch_optional(state.db())
		.await?
		.ok_or(Error::NoContent)?;

	debug!("Player:\n\t{player:?}");

	Ok(Player::try_from(player)?.into())
}
