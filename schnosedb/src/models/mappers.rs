#[derive(
	Debug, Clone, Copy, PartialEq, Eq, sqlx::FromRow, serde::Serialize, serde::Deserialize,
)]
pub struct MapperRow {
	pub map_id: u16,
	pub mapper_id: u32,
}
