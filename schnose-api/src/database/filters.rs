use {
	crate::{Error, Result},
	serde::{Deserialize, Serialize},
	sqlx::FromRow,
	utoipa::ToSchema,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct FilterRow {
	pub course_id: i32,
	pub mode_id: i16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Filter {
	pub map_id: u16,
	pub map_name: String,
	pub stage: u8,
	pub kzt: bool,
	pub skz: bool,
	pub vnl: bool,
}

impl TryFrom<FilterRow> for Filter {
	type Error = Error;

	#[tracing::instrument(level = "TRACE", err(Debug))]
	fn try_from(row: FilterRow) -> Result<Self> {
		unimplemented!();
	}
}
