use {
	super::{Course, CourseRow, Player, PlayerRow},
	crate::{Error, Result},
	color_eyre::eyre::Context,
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
pub struct MapRow {
	pub id: i16,
	pub name: String,
	pub global: bool,
	pub courses: SqlJson<Vec<CourseRow>>,
	pub workshop_id: Option<i64>,
	pub filesize: Option<i64>,
	pub mappers: Option<SqlJson<Vec<PlayerRow>>>,
	pub created_on: DateTime<Utc>,
	pub updated_on: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct MapModel {
	pub id: u16,
	pub name: String,
	pub global: bool,
	pub courses: Vec<Course>,
	pub workshop_id: Option<u32>,
	pub filesize: Option<u64>,
	pub mappers: Option<Vec<Player>>,
	pub created_on: DateTime<Utc>,
	pub updated_on: DateTime<Utc>,
}

impl TryFrom<MapRow> for MapModel {
	type Error = Error;

	#[tracing::instrument(level = "TRACE", err(Debug))]
	fn try_from(row: MapRow) -> Result<Self> {
		Ok(Self {
			id: row.id.try_into().context("Found negative MapID.")?,
			name: row.name,
			global: row.global,
			courses: row
				.courses
				.0
				.into_iter()
				.map(TryInto::try_into)
				.collect::<Result<Vec<_>>>()
				.context("Found invalid course in database.")?,
			workshop_id: if let Some(id) = row.workshop_id {
				Some(id.try_into().context("Found negative WorkshopID.")?)
			} else {
				None
			},
			filesize: if let Some(filesize) = row.workshop_id {
				Some(filesize.try_into().context("Found negative filesize.")?)
			} else {
				None
			},
			mappers: if let Some(mappers) = row.mappers {
				Some(
					mappers
						.0
						.into_iter()
						.map(TryInto::try_into)
						.collect::<Result<Vec<_>>>()
						.context("Found invalid player in database.")?,
				)
			} else {
				None
			},
			created_on: row.created_on,
			updated_on: row.updated_on,
		})
	}
}
