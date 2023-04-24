use {
	schnosedb::models::ModeRow,
	serde::{Deserialize, Serialize},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mode {
	pub id: u8,
	pub name: String,
	pub abbr: String,
	pub long: String,
}

impl From<ModeRow> for Mode {
	fn from(value: ModeRow) -> Self {
		let mode = gokz_rs::Mode::try_from(value.id).expect("Got unknown mode from database.");
		Self {
			id: mode as u8,
			name: mode.api(),
			abbr: mode.short(),
			long: mode.to_string(),
		}
	}
}
