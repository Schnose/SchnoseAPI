use {
	crate::{
		database::{Server, ServerRow},
		models::app_state::AppState,
		Error, Result,
	},
	axum::{
		extract::{Path, State},
		http, Json,
	},
	color_eyre::eyre::Context,
	gokz_rs::types::ServerIdentifier,
	sqlx::QueryBuilder,
	std::sync::Arc,
};

#[utoipa::path(
	get,
	tag = "Servers",
	path = "/api/servers/{ident}",
	responses(
		(status = 200, body = Server),
		(status = 400, description = "An invalid server identifier was provided."),
		(status = 500, body = Error),
	),
	params(
		("ident" = String, Path, description = "The server's name or ID."),
	),
)]
#[tracing::instrument(level = "DEBUG", skip(state), err(Debug))]
pub async fn ident(
	method: http::Method,
	Path(server_identifier): Path<ServerIdentifier>,
	State(state): State<Arc<AppState>>,
) -> Result<Json<Server>> {
	let mut query = QueryBuilder::new(
		r#"
		SELECT
			server.id,
			server.name,
			ROW_TO_JSON(owner) owned_by
		FROM servers server
		JOIN players owner
			ON owner.id = server.owned_by
		WHERE
		"#,
	);

	match server_identifier {
		ServerIdentifier::Id(server_id) => query.push(" server.id = ").push_bind(server_id as i16),
		ServerIdentifier::Name(server_name) => {
			query.push(" server.name ILIKE ").push_bind(format!("%{server_name}%"))
		}
	};

	let server = query
		.build_query_as::<ServerRow>()
		.fetch_optional(state.db())
		.await
		.context("Failed to fetch server from database.")?
		.ok_or(Error::NoContent)?
		.try_into()
		.context("Found invalid server in database.")?;

	Ok(Json(server))
}
