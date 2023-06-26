use {
	serde::{Deserialize, Serialize},
	sqlx::{
		types::chrono::{DateTime, Utc},
		FromRow,
	},
	utoipa::ToSchema,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Record {
	pub id: i32,
	pub course_id: i32,
	pub mode_id: i16,
	pub player_id: i32,
	pub server_id: i16,
	pub time: f64,
	pub teleports: i16,
	pub created_on: DateTime<Utc>,
}
