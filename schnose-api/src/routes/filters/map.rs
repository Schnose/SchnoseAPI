use {
	crate::{database::Filter, models::app_state::AppState, Error, Result},
	axum::{
		extract::{Path, State},
		http, Json,
	},
	color_eyre::eyre::Context,
	gokz_rs::types::MapIdentifier,
	std::{collections::HashMap, sync::Arc},
};

#[utoipa::path(
	get,
	tag = "Filters",
	path = "/filters/map/{ident}",
	responses(
		(status = 200, body = Vec<Filter>),
		(status = 400, description = "An invalid map identifier was provided."),
		(status = 500, body = Error),
	),
	params(
		("ident" = String, Path, description = "The map's name or ID."),
	),
)]
#[tracing::instrument(level = "DEBUG", skip(state), err(Debug))]
pub async fn map(
	method: http::Method,
	Path(map_identifier): Path<MapIdentifier>,
	State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Filter>>> {
	let map_id = match map_identifier {
		MapIdentifier::Id(map_id) => map_id as i16,
		MapIdentifier::Name(map_name) => {
			sqlx::query!("SELECT id FROM maps WHERE name LIKE $1", format!("%{map_name}%"))
				.fetch_optional(state.db())
				.await
				.context("Failed to fetch map from database.")?
				.ok_or(Error::NoContent)?
				.id
		}
	};

	let filter_rows = sqlx::query!(
		r#"
		SELECT
			m.name map_name,
			c.stage,
			f.mode_id
		FROM filters f
		JOIN courses c ON c.id = f.course_id
		JOIN maps m ON m.id = c.map_id
		WHERE map_id = $1
		"#,
		map_id
	)
	.fetch_all(state.db())
	.await
	.context("Failed to fetch filters from database.")?;

	if filter_rows.is_empty() {
		return Err(Error::NoContent);
	}

	let mut filters = HashMap::<i16, Filter>::new();

	for filter in filter_rows {
		filters
			.entry(filter.stage)
			.and_modify(|f| {
				match filter.mode_id {
					200 => f.kzt = true,
					201 => f.skz = true,
					202 => f.vnl = true,
					_ => unreachable!(),
				};
			})
			.or_insert(Filter {
				map_id: map_id as u16,
				map_name: filter.map_name,
				stage: u8::try_from(filter.stage).context("Found invalid stage in database.")?,
				kzt: filter.mode_id == 200,
				skz: filter.mode_id == 201,
				vnl: filter.mode_id == 202,
			});
	}

	Ok(Json(filters.into_values().collect()))
}
