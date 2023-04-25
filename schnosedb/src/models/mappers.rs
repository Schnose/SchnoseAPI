use {
	serde::{Deserialize, Serialize},
	sqlx::FromRow,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRow, Serialize, Deserialize)]
pub struct MapperRow {
	pub map_id: u16,
	pub mapper_id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, FromRow, Serialize, Deserialize)]
pub struct JoinedMapperRow {
	pub map_id: u16,
	pub mapper_id: u32,
	pub mapper_name: String,
}
