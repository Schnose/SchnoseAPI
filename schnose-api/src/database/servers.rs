use {
	super::{Player, PlayerRow},
	crate::{Error, Result},
	color_eyre::eyre::Context,
	serde::{Deserialize, Serialize},
	sqlx::{types::Json as SqlJson, FromRow},
	utoipa::ToSchema,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct ServerRow {
	pub id: i16,
	pub name: String,
	pub owned_by: SqlJson<PlayerRow>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Server {
	pub id: u16,
	pub name: String,
	pub owned_by: Player,
}

impl TryFrom<ServerRow> for Server {
	type Error = Error;

	#[tracing::instrument(level = "TRACE", err(Debug))]
	fn try_from(row: ServerRow) -> Result<Self> {
		Ok(Self {
			id: row.id.try_into().context("Found negative ServerID.")?,
			name: row.name,
			owned_by: row.owned_by.0.try_into().context("Found invalid player in database.")?,
		})
	}
}
