use {
	crate::{response::Response, state::APIState},
	axum::extract::{Query, State},
	itertools::Itertools,
	schnose_api::models::Player,
	schnosedb::models::PlayerRow,
	serde::Deserialize,
	sqlx::QueryBuilder,
	tracing::{debug, trace},
};

#[derive(Debug, Deserialize)]
pub struct Params {
	is_banned: Option<bool>,
	limit: Option<u16>,
	offset: Option<i64>,
}

#[axum::debug_handler]
pub async fn get(
	Query(params): Query<Params>,
	State(state): State<APIState>,
) -> Response<Vec<Player>> {
	trace!("GET /api/players");
	trace!("{params:?}");

	let mut query = QueryBuilder::new("SELECT * FROM players WHERE id > 0");

	if let Some(is_banned) = params.is_banned {
		query
			.push(" AND is_banned = ")
			.push_bind(is_banned as u8);
	}

	query
		.push(" LIMIT ")
		.push_bind(match params.limit.unwrap_or(100) {
			0 => 1,
			limit @ (1..=500) => limit,
			501.. => 500,
		});

	if let Some(offset) = params.offset {
		query.push(" OFFSET ").push_bind(offset);
	}

	let players: Vec<PlayerRow> = query
		.build_query_as()
		.fetch_all(state.db())
		.await?;

	debug!("Players:\n\t{players:?}");

	Ok(players
		.into_iter()
		.flat_map(TryInto::try_into)
		.inspect(|player| debug!("Parsed player: {player:?}"))
		.collect_vec()
		.into())
}
