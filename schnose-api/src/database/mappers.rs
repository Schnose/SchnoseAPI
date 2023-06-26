use {
	crate::{Error, Result},
	gokz_rs::types::SteamID,
	serde::{Deserialize, Serialize},
	sqlx::FromRow,
	utoipa::ToSchema,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct MapperRow {
	pub player_id: i32,
	pub map_id: i16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Mapper {
	pub player: SteamID,
	pub map_id: u16,
}

impl TryFrom<MapperRow> for Mapper {
	type Error = Error;

	#[tracing::instrument(level = "TRACE", err(Debug))]
	fn try_from(row: MapperRow) -> Result<Self> {
		Ok(Self {
			player: u32::try_from(row.player_id)?.try_into()?,
			map_id: row.map_id.try_into()?,
		})
	}
}
