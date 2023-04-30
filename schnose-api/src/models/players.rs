use {
	crate::{
		error::{yeet, Error, Result},
		serde::Bool,
	},
	gokz_rs::SteamID,
	schnosedb::models::PlayerRow,
	serde::{Deserialize, Serialize},
	sqlx::FromRow,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
	pub name: String,
	pub steam_id: SteamID,
	pub is_banned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, FromRow)]
pub struct PlayerQuery {
	pub id: u32,
	pub name: String,
	pub is_banned: Bool,
}

impl TryFrom<PlayerRow> for Player {
	type Error = Error;

	fn try_from(value: PlayerRow) -> Result<Self> {
		if value.id == 0 {
			yeet!("SteamID must not be 0.");
		}

		Ok(Self {
			name: value.name,
			steam_id: SteamID::from_id32(value.id),
			is_banned: value.is_banned,
		})
	}
}
