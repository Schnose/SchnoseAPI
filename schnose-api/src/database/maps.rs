use {
	crate::{Error, Result},
	gokz_rs::types::SteamID,
	serde::{Deserialize, Serialize},
	sqlx::{
		types::chrono::{DateTime, Utc},
		FromRow,
	},
	utoipa::ToSchema,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct MapRow {
	pub id: i16,
	pub name: String,
	pub global: bool,
	pub workshop_id: Option<i32>,
	pub filesize: Option<i64>,
	pub approved_by: Option<i32>,
	pub created_on: DateTime<Utc>,
	pub updated_on: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Map {
	pub id: u16,
	pub name: String,
	pub global: bool,
	pub workshop_id: Option<u32>,
	pub filesize: Option<u64>,
	pub approved_by: Option<SteamID>,
	pub created_on: DateTime<Utc>,
	pub updated_on: DateTime<Utc>,
}

impl TryFrom<MapRow> for Map {
	type Error = Error;

	#[tracing::instrument(level = "TRACE", err(Debug))]
	fn try_from(row: MapRow) -> Result<Self> {
		Ok(Self {
			id: row.id.try_into()?,
			name: row.name,
			global: row.global,
			workshop_id: if let Some(id) = row.workshop_id { Some(id.try_into()?) } else { None },
			filesize: if let Some(filesize) = row.workshop_id {
				Some(filesize.try_into()?)
			} else {
				None
			},
			approved_by: if let Some(steam_id) = row.workshop_id {
				Some(u32::try_from(steam_id)?.try_into()?)
			} else {
				None
			},
			created_on: row.created_on,
			updated_on: row.updated_on,
		})
	}
}
