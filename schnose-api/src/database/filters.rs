use {
	crate::{Error, Result},
	gokz_rs::types::Mode,
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
	pub course_id: u32,
	pub mode: Mode,
}

impl TryFrom<FilterRow> for Filter {
	type Error = Error;

	#[tracing::instrument(level = "TRACE", err(Debug))]
	fn try_from(row: FilterRow) -> Result<Self> {
		Ok(Self {
			course_id: row.course_id.try_into()?,
			mode: row.mode_id.try_into()?,
		})
	}
}
