pub mod ident;
pub use ident::ident;
use {
	crate::{
		database::players::{Player, PlayerRow},
		models::app_state::AppState,
		Error, Result,
	},
	axum::{
		extract::{Query, State},
		http, Json,
	},
	color_eyre::eyre::Context,
	serde::Deserialize,
	sqlx::QueryBuilder,
	std::sync::Arc,
	utoipa::{IntoParams, ToSchema},
};

#[derive(Debug, Clone, Deserialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct Params {
	pub name: Option<String>,
	pub is_banned: Option<bool>,
	pub offset: Option<i64>,
	pub limit: Option<u64>,
}

#[utoipa::path(
	get,
	tag = "Players",
	path = "/players",
	responses(
		(status = 200, body = Vec<Player>),
		(status = 204, body = ()),
		(status = 500, body = Error),
	),
	params(Params),
)]
#[tracing::instrument(level = "DEBUG", skip(state), err(Debug))]
pub async fn root(
	method: http::Method,
	Query(Params {
		name,
		is_banned,
		offset,
		limit,
	}): Query<Params>,
	State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Player>>> {
	let mut query = QueryBuilder::new("SELECT * FROM players");

	let mut filter = " WHERE ";

	if let Some(name) = name {
		query.push(filter).push("name ILIKE ").push_bind(format!("%{name}%"));
		filter = " AND ";
	}

	if let Some(is_banned) = is_banned {
		query.push(filter).push("is_banned = ").push_bind(is_banned);
	}

	query
		.push(" LIMIT ")
		.push_bind(limit.map_or(100, |limit| (limit as i64).min(1000)))
		.push(" OFFSET ")
		.push_bind(offset.unwrap_or(0));

	let players = query
		.build_query_as::<PlayerRow>()
		.fetch_all(state.db())
		.await
		.context("Failed to fetch players from database.")?
		.into_iter()
		.map(TryInto::try_into)
		.collect::<Result<Vec<Player>>>()
		.context("Found invalid player in database.")?;

	if players.is_empty() {
		return Err(Error::NoContent);
	}

	Ok(Json(players))
}
