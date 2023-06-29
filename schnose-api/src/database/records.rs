use {
	super::{Course, CourseRow},
	crate::{Error, Result},
	color_eyre::eyre::Context,
	gokz_rs::types::{Mode, SteamID},
	serde::{Deserialize, Serialize},
	sqlx::{
		types::{
			chrono::{DateTime, Utc},
			Json as SqlJson,
		},
		FromRow,
	},
	utoipa::ToSchema,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct RecordRow {
	pub id: i32,
	pub course: SqlJson<CourseRow>,
	pub map_name: String,
	pub mode_id: i16,
	pub player_id: i64,
	pub player_name: String,
	pub server_id: i16,
	pub server_name: String,
	pub time: f64,
	pub teleports: i16,
	pub created_on: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Record {
	pub id: u32,
	pub course: Course,
	#[schema(value_type = String)]
	pub mode: Mode,
	#[schema(value_type = String)]
	pub steam_id: SteamID,
	pub player_name: String,
	pub server_id: u16,
	pub server_name: String,
	pub time: f64,
	pub teleports: u16,
	pub created_on: DateTime<Utc>,
}

impl TryFrom<RecordRow> for Record {
	type Error = Error;

	#[tracing::instrument(level = "TRACE", err(Debug))]
	fn try_from(row: RecordRow) -> Result<Self> {
		Ok(Self {
			id: row.id.try_into().context("Found negative RecordID.")?,
			course: row.course.0.try_into().context("Found invalid course in database.")?,
			mode: row.mode_id.try_into().context("Found invalid ModeID.")?,
			steam_id: u32::try_from(row.player_id)
				.context("Found negative SteamID.")?
				.try_into()
				.context("Found invalid SteamID.")?,
			player_name: row.player_name,
			server_id: row.server_id.try_into().context("Found negative ServerID.")?,
			server_name: row.server_name,
			time: row.time,
			teleports: row.teleports.try_into().context("Found negative teleports.")?,
			created_on: row.created_on,
		})
	}
}
