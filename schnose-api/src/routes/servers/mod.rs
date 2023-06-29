pub mod ident;
pub use ident::ident;
use {
	crate::{
		database::{Server, ServerRow},
		models::app_state::AppState,
		Error, Filter, Result,
	},
	axum::{
		extract::{Query, State},
		http, Json,
	},
	color_eyre::eyre::Context,
	gokz_rs::types::PlayerIdentifier,
	serde::Deserialize,
	sqlx::QueryBuilder,
	std::sync::Arc,
	utoipa::IntoParams,
};

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct Params {
	pub name: Option<String>,
	#[param(value_type = Option<String>)]
	pub owned_by: Option<PlayerIdentifier>,
	pub offset: Option<i64>,
	pub limit: Option<u64>,
}

#[utoipa::path(
	get,
	tag = "Servers",
	path = "/servers",
	responses(
		(status = 200, body = Vec<Server>),
		(status = 204),
		(status = 400, body = Error),
		(status = 500, body = Error),
	),
	params(Params),
)]
#[tracing::instrument(level = "DEBUG", skip(state), err(Debug))]
pub async fn root(
	method: http::Method,
	Query(Params {
		name,
		owned_by,
		offset,
		limit,
	}): Query<Params>,
	State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Server>>> {
	let mut query = QueryBuilder::new(
		r#"
		SELECT
			server.id,
			server.name,
			ROW_TO_JSON(owner) owned_by
		FROM servers server
		JOIN players owner
			ON owner.id = server.owned_by
		"#,
	);

	let mut filter = Filter::Where;

	if let Some(name) = name {
		query.push(filter).push("server.name ILIKE ").push_bind(format!("%{name}%"));
		filter.and();
	}

	if let Some(owned_by) = owned_by {
		query.push(filter);

		match owned_by {
			PlayerIdentifier::SteamID(steam_id) => {
				query.push("owner.id = ").push_bind(steam_id.community_id() as i64);
			}

			PlayerIdentifier::Name(name) => {
				query.push("owner.name ILIKE ").push_bind(format!("%{name}%"));
			}
		};
	}

	query
		.push(" LIMIT ")
		.push_bind(limit.map_or(1000, |limit| (limit as i64).min(2000)))
		.push(" OFFSET ")
		.push_bind(offset.unwrap_or(0));

	let servers = query
		.build_query_as::<ServerRow>()
		.fetch_all(state.db())
		.await
		.context("Failed to fetch servers from database.")?
		.into_iter()
		.map(TryInto::try_into)
		.collect::<Result<Vec<_>>>()
		.context("Found invalid server in database.")?;

	if servers.is_empty() {
		return Err(Error::NoContent);
	}

	Ok(Json(servers))
}
