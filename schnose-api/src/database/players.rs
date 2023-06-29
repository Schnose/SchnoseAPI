use {
	crate::{Error, Result},
	color_eyre::eyre::Context,
	gokz_rs::types::SteamID,
	serde::{Deserialize, Serialize},
	sqlx::FromRow,
	utoipa::ToSchema,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct PlayerRow {
	pub id: i64,
	pub name: String,
	pub is_banned: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Player {
	#[schema(value_type = String)]
	pub steam_id: SteamID,
	pub name: String,
	pub is_banned: bool,
}

impl TryFrom<PlayerRow> for Player {
	type Error = Error;

	#[tracing::instrument(level = "TRACE", err(Debug))]
	fn try_from(row: PlayerRow) -> Result<Self> {
		Ok(Self {
			steam_id: u32::try_from(row.id)
				.context("Found negative SteamID.")?
				.try_into()
				.context("Found invalid SteamID.")?,
			name: row.name,
			is_banned: row.is_banned,
		})
	}
}
