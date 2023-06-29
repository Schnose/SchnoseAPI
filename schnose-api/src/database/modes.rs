use {
	crate::{Error, Result},
	color_eyre::eyre::Context,
	serde::{Deserialize, Serialize},
	sqlx::FromRow,
	utoipa::ToSchema,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct ModeRow {
	pub id: i16,
	pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Mode {
	pub id: u8,
	pub name: String,
}

impl TryFrom<ModeRow> for Mode {
	type Error = Error;

	#[tracing::instrument(level = "TRACE", err(Debug))]
	fn try_from(row: ModeRow) -> Result<Self> {
		Ok(Self {
			id: row.id.try_into().context("Failed to convert ModeID.")?,
			name: row.name,
		})
	}
}
