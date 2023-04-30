use {
	crate::{response::Response, state::APIState},
	axum::extract::{Path, State},
	gokz_rs::{ServerIdentifier, SteamID},
	schnose_api::{
		error::Error,
		models::{Server, ServerOwner, ServerQuery},
	},
	sqlx::QueryBuilder,
	tracing::{debug, trace},
};

#[axum::debug_handler]
pub async fn get(
	Path(server): Path<ServerIdentifier>,
	State(state): State<APIState>,
) -> Response<Server> {
	trace!("GET /api/servers/{server:?}");

	let mut query = QueryBuilder::new(
		r#"
		SELECT
		  server.*,
		  JSON_OBJECT(
		    "name",     owner.name,
		    "steam_id", owner.id,
		  ) AS owned_by,
		  approver.id AS approved_by
		FROM servers AS server
		LEFT JOIN players AS owner ON player.id = server.owned_by
		LEFT JOIN players AS approver ON player.id = server.approved_by
		"#,
	);

	match server {
		ServerIdentifier::Name(server_name) => {
			query
				.push(" WHERE server.name LIKE ")
				.push_bind(format!("%{server_name}%"));
		}
		ServerIdentifier::ID(server_id) => {
			query
				.push(" WHERE server.id = ")
				.push_bind(server_id);
		}
	}

	query.push(" LIMIT 1 ");

	let server: ServerQuery = query
		.build_query_as()
		.fetch_optional(state.db())
		.await?
		.ok_or(Error::NoContent)?;

	debug!("Server:\n\t{server:?}");

	Ok(Server {
		id: server.id,
		name: server.name,
		owned_by: server
			.owned_by
			.0
			.and_then(|owner| ServerOwner::try_from(owner).ok()),
		approved_by: server
			.approved_by
			.and_then(|id| (id != 0).then_some(SteamID::from_id32(id))),
	}
	.into())
}
