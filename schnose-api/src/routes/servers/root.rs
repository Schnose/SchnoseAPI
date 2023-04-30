use {
	crate::{response::Response, state::APIState},
	axum::extract::{Query, State},
	gokz_rs::{PlayerIdentifier, SteamID},
	itertools::Itertools,
	schnose_api::{
		error::{yeet, Error},
		models::{Server, ServerOwner, ServerQuery},
	},
	serde::Deserialize,
	sqlx::QueryBuilder,
	tracing::{debug, trace},
};

#[derive(Debug, Deserialize)]
pub struct Params {
	name: Option<String>,
	owned_by: Option<PlayerIdentifier>,
	limit: Option<u16>,
}

#[axum::debug_handler]
pub async fn get(
	Query(params): Query<Params>,
	State(state): State<APIState>,
) -> Response<Vec<Server>> {
	trace!("GET /api/servers");
	trace!("{params:?}");

	let mut query = QueryBuilder::new(
		r#"
		SELECT
		  server.*,
		  JSON_OBJECT(
		    "name",     owner.name,
		    "steam_id", owner.id
		  ) AS owned_by,
		  approver.id AS approved_by
		FROM servers AS server
		LEFT JOIN players AS owner ON owner.id = server.owned_by
		LEFT JOIN players AS approver ON approver.id = server.approved_by
		"#,
	);

	if let Some(name) = params.name {
		query
			.push(" WHERE server.name LIKE ")
			.push_bind(format!("%{name}%"));
	}

	if let Some(owner) = params.owned_by {
		match owner {
			PlayerIdentifier::Name(name) => {
				query
					.push(" WHERE owner.name LIKE ")
					.push_bind(format!("%{name}%"));
			}
			PlayerIdentifier::SteamID(steam_id) => {
				query
					.push(" WHERE owner.id = ")
					.push_bind(steam_id.as_id32());
			}
		}
	}

	query
		.push(" LIMIT ")
		.push_bind(match params.limit.unwrap_or(1500) {
			0 => 1,
			limit @ (1..=1500) => limit,
			1501.. => 1500,
		});

	let servers: Vec<ServerQuery> = query
		.build_query_as()
		.fetch_all(state.db())
		.await?;

	debug!("Servers:\n\t{servers:?}");

	if servers.is_empty() {
		yeet!(Error::NoContent);
	}

	let servers = servers
		.into_iter()
		.map(|server| Server {
			id: server.id,
			name: server.name,
			owned_by: server
				.owned_by
				.0
				.and_then(|owner| ServerOwner::try_from(owner).ok()),
			approved_by: server
				.approved_by
				.and_then(|id| (id != 0).then_some(SteamID::from_id32(id))),
		})
		.collect_vec();

	Ok(servers.into())
}
