#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct ModeRow {
	pub id: u8,
	pub name: String,
}
