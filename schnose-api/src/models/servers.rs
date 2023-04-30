use {
	super::{Mapper, MapperQuery},
	gokz_rs::SteamID,
	serde::{Deserialize, Serialize},
	sqlx::{types::Json, FromRow},
};

pub type ServerOwner = Mapper;
pub type ServerOwnerQuery = MapperQuery;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Server {
	pub id: u16,
	pub name: String,
	pub owned_by: Option<ServerOwner>,
	pub approved_by: Option<SteamID>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct ServerQuery {
	pub id: u16,
	pub name: String,
	pub owned_by: Json<Option<ServerOwnerQuery>>,
	pub approved_by: Option<u32>,
}
