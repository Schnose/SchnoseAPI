use {
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
pub struct Mapper {
	pub name: String,
	pub steam_id: SteamID,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapperQuery {
	pub name: String,
	pub steam_id: u32,
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
	pub courses: Json<Vec<Course>>,
	pub mappers: Json<Vec<MapperQuery>>,
	pub approved_by: Option<u32>,

	#[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
	pub created_on: DateTime<Utc>,

	#[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
	pub updated_on: DateTime<Utc>,
}
