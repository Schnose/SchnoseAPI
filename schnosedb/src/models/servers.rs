#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct ServerRow {
	pub id: u16,
	pub name: String,
	pub owned_by: u32,
	pub approved_by: u32,
}
