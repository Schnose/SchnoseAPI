#[derive(
	Debug, Clone, Copy, PartialEq, Eq, sqlx::FromRow, serde::Serialize, serde::Deserialize,
)]
pub struct FilterRow {
	pub course_id: u32,
	pub mode_id: u8,
}
