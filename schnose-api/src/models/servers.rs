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

impl From<ServerQuery> for Server {
	fn from(value: ServerQuery) -> Self {
		Self {
			id: value.id,
			name: value.name,
			owned_by: value
				.owned_by
				.0
				.and_then(|owner| ServerOwner::try_from(owner).ok()),
			approved_by: value
				.approved_by
				.and_then(|id| (id != 0).then_some(SteamID::from_id32(id))),
		}
	}
}
