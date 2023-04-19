use sqlx::types::chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct MapRow {
	pub id: u16,
	pub name: String,
	pub global: bool,
	pub filesize: u32,
	pub approved_by: u32,

	#[serde(
		serialize_with = "crate::serialize_datetime",
		deserialize_with = "crate::deserialize_datetime"
	)]
	pub created_on: DateTime<Utc>,

	#[serde(
		serialize_with = "crate::serialize_datetime",
		deserialize_with = "crate::deserialize_datetime"
	)]
	pub updated_on: DateTime<Utc>,
}
