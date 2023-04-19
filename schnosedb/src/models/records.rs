use sqlx::types::chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct RecordRow {
	pub id: u32,
	pub course_id: u32,
	pub mode_id: u8,
	pub player_id: u32,
	pub server_id: u16,
	pub time: f64,
	pub teleports: u16,

	#[serde(
		serialize_with = "crate::serialize_datetime",
		deserialize_with = "crate::deserialize_datetime"
	)]
	pub created_on: DateTime<Utc>,
}
