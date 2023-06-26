use {
	crate::{Error, Result},
	gokz_rs::types::SteamID,
	serde::{Deserialize, Serialize},
	sqlx::FromRow,
	utoipa::ToSchema,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct ServerRow {
	pub id: i16,
	pub name: String,
	pub owned_by: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Server {
	pub id: u16,
	pub name: String,
	pub owned_by: SteamID,
}

impl TryFrom<ServerRow> for Server {
	type Error = Error;

	#[tracing::instrument(level = "TRACE", err(Debug))]
	fn try_from(row: ServerRow) -> Result<Self> {
		Ok(Self {
			id: row.id.try_into()?,
			name: row.name,
			owned_by: u32::try_from(row.owned_by)?.try_into()?,
		})
	}
}
