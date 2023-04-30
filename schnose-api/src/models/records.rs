use {
	super::{Course, CourseQuery, Player, PlayerQuery, Server, ServerQuery},
	crate::error::{Error, Result},
	gokz_rs::{Mode, SteamID},
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Record {
	pub id: u32,
	pub map_id: u16,
	pub map_name: String,
	pub course: Course,
	pub mode: Mode,
	pub player: Player,
	pub server: Server,
	pub time: f64,
	pub teleports: u16,

	#[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
	pub created_on: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, FromRow)]
pub struct RecordQuery {
	pub id: u32,
	pub map_id: u16,
	pub map_name: String,
	pub course: Json<CourseQuery>,
	pub mode_id: u8,
	pub player: Json<PlayerQuery>,
	pub server: Json<ServerQuery>,
	pub time: f64,
	pub teleports: u16,

	#[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
	pub created_on: DateTime<Utc>,
}

impl TryFrom<RecordQuery> for Record {
	type Error = Error;

	fn try_from(value: RecordQuery) -> Result<Self> {
		Ok(Self {
			id: value.id,
			map_id: value.map_id,
			map_name: value.map_name,
			course: value.course.0.try_into()?,
			mode: value
				.mode_id
				.try_into()
				.expect("Modes in the database should always have valid IDs."),
			player: Player {
				name: value.player.0.name,
				steam_id: SteamID::from_id32(value.player.0.id),
				is_banned: value
					.player
					.0
					.is_banned
					.try_into()
					.expect("booleans from the database should always be 0 or 1"),
			},
			server: value.server.0.into(),
			time: value.time,
			teleports: value.teleports,
			created_on: value.created_on,
		})
	}
}
