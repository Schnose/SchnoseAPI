use {
	crate::{Error, Result},
	gokz_rs::types::{Mode, SteamID},
	serde::{Deserialize, Serialize},
	sqlx::{
		types::chrono::{DateTime, Utc},
		FromRow,
	},
	utoipa::ToSchema,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct RecordRow {
	pub id: i32,
	pub course_id: i32,
	pub mode_id: i16,
	pub player_id: i64,
	pub server_id: i16,
	pub time: f64,
	pub teleports: i16,
	pub created_on: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Record {
	pub id: u32,
	pub course_id: u32,
	pub mode: Mode,
	pub player: SteamID,
	pub server_id: u16,
	pub time: f64,
	pub teleports: u16,
	pub created_on: DateTime<Utc>,
}

impl TryFrom<RecordRow> for Record {
	type Error = Error;

	#[tracing::instrument(level = "TRACE", err(Debug))]
	fn try_from(row: RecordRow) -> Result<Self> {
		Ok(Self {
			id: row.id.try_into()?,
			course_id: row.course_id.try_into()?,
			mode: row.mode_id.try_into()?,
			player: u32::try_from(row.player_id)?.try_into()?,
			server_id: row.server_id.try_into()?,
			time: row.time,
			teleports: row.teleports.try_into()?,
			created_on: row.created_on,
		})
	}
}