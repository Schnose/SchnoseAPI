use {
	crate::error::{Error, Result},
	gokz_rs::{SteamID, Tier},
	schnosedb::{deserialize_datetime, serialize_datetime},
	serde::{Deserialize, Serialize},
	sqlx::{
		types::{
			chrono::{DateTime, Utc},
			Json,
		},
		FromRow,
	},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Course {
	pub id: u32,
	pub stage: u8,
	pub tier: Tier,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CourseQuery {
	pub id: Option<u32>,
	pub stage: Option<u8>,
	pub tier: Option<Tier>,
}

impl TryFrom<CourseQuery> for Course {
	type Error = Error;

	fn try_from(value: CourseQuery) -> Result<Self> {
		match (value.id, value.stage, value.tier) {
			(Some(id), Some(stage), Some(tier)) => Ok(Self { id, stage, tier }),
			_ => Err(Error::NoContent),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mapper {
	pub name: String,
	pub steam_id: SteamID,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapperQuery {
	pub name: Option<String>,
	pub steam_id: Option<u32>,
}

impl TryFrom<MapperQuery> for Mapper {
	type Error = Error;

	fn try_from(value: MapperQuery) -> Result<Self> {
		match (value.name, value.steam_id) {
			(Some(name), Some(steam_id)) if steam_id != 0 => Ok(Self {
				name,
				steam_id: SteamID::from_id32(steam_id),
			}),
			_ => Err(Error::NoContent),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Map {
	pub id: u16,
	pub name: String,
	pub global: bool,
	pub filesize: u32,
	pub courses: Vec<Course>,
	pub mappers: Vec<Mapper>,
	pub approved_by: Option<SteamID>,

	#[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
	pub created_on: DateTime<Utc>,

	#[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
	pub updated_on: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct MapQuery {
	pub id: u16,
	pub name: String,
	pub global: bool,
	pub filesize: u32,
	pub courses: Json<Vec<CourseQuery>>,
	pub mappers: Json<Vec<MapperQuery>>,
	pub approved_by: Option<u32>,

	#[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
	pub created_on: DateTime<Utc>,

	#[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
	pub updated_on: DateTime<Utc>,
}
