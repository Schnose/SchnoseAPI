use {
	crate::{Error, Result},
	color_eyre::eyre::Context,
	gokz_rs::types::Tier,
	serde::{Deserialize, Serialize},
	sqlx::FromRow,
	utoipa::ToSchema,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct CourseRow {
	pub id: i32,
	pub map_id: i16,
	pub stage: i16,
	pub tier: Option<i16>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Course {
	pub id: u32,
	pub map_id: u16,
	pub stage: u8,
	pub tier: Option<Tier>,
}

impl TryFrom<CourseRow> for Course {
	type Error = Error;

	#[tracing::instrument(level = "TRACE", err(Debug))]
	fn try_from(row: CourseRow) -> Result<Self> {
		Ok(Self {
			id: row.id.try_into().context("Found negative CourseID.")?,
			map_id: row.map_id.try_into().context("Found negative MapID.")?,
			stage: row.stage.try_into().context("Found negative stage.")?,
			tier: if let Some(tier) = row.tier {
				Some(tier.try_into().context("Found invalid Tier.")?)
			} else {
				None
			},
		})
	}
}
