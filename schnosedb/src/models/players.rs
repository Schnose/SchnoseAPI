#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct PlayerRow {
	pub id: u32,
	pub name: String,
	pub is_banned: bool,
}
