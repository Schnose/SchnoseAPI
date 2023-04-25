use {
	crate::error::{yeet, Error, Result},
	gokz_rs::SteamID,
	schnosedb::models::PlayerRow,
	serde::{Deserialize, Serialize},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
	pub name: String,
	pub steam_id: SteamID,
	pub is_banned: bool,
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
