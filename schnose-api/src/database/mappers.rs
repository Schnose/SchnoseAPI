use {
	crate::{Error, Result},
	color_eyre::eyre::Context,
	gokz_rs::types::SteamID,
	serde::{Deserialize, Serialize},
	sqlx::FromRow,
	utoipa::ToSchema,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct MapperRow {
	pub player_id: i64,
	pub map_id: i16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Mapper {
	#[schema(value_type = String)]
	pub player: SteamID,
	pub map_id: u16,
}

impl TryFrom<MapperRow> for Mapper {
	type Error = Error;

	#[tracing::instrument(level = "TRACE", err(Debug))]
	fn try_from(row: MapperRow) -> Result<Self> {
		Ok(Self {
			player: u32::try_from(row.player_id)
				.context("Found negative SteamID.")?
				.try_into()
				.context("Found invalid SteamID.")?,
			map_id: row.map_id.try_into().context("Found negative MapID.")?,
		})
	}
}
