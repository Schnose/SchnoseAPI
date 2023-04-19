#[derive(
	Debug, Clone, Copy, PartialEq, Eq, sqlx::FromRow, serde::Serialize, serde::Deserialize,
)]
pub struct CourseRow {
	pub id: u32,
	pub map_id: u16,
	pub stage: u8,
	pub tier: u8,
}
