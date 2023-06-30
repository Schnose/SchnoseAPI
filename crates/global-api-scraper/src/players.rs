use {
	color_eyre::{
		eyre::{Context, ContextCompat},
		Result,
	},
	gokz_rs::global_api,
	sqlx::{PgPool, QueryBuilder},
	tracing::{info, trace},
};

#[tracing::instrument(level = "TRACE", err(Debug))]
pub async fn fetch_players(
	start_offset: u32,
	backwards: bool,
	chunk_size: usize,
	gokz_client: &gokz_rs::Client,
	pool: &PgPool,
) -> Result<usize> {
	let mut total = 0;

	let mut params = global_api::players::Params {
		offset: Some(start_offset as i32),
		limit: Some(chunk_size as u32),
		..Default::default()
	};

	loop {
		let Ok(players) = global_api::players::root(&params, gokz_client).await else {
			info!("No players anymore, done.");
			break Ok(total);
		};

		if players.is_empty() {
			info!("No players anymore, done.");
			break Ok(total);
		}

		trace!(amount = %players.len(), "Fetched players.");
		total += players.len();

		let mut query = QueryBuilder::new(
			r#"
			INSERT INTO players
				(id, name, is_banned)
			"#,
		);

		query
			.push_values(players, |mut query, player| {
				query
					.push_bind(player.steam_id.community_id() as i64)
					.push_bind(player.name)
					.push_bind(player.is_banned);
			})
			.push(" ON CONFLICT DO NOTHING ");

		query.build().execute(pool).await.context("Failed to insert players into the database.")?;

		trace!("Inserted players into database.");

		let offset =
			params.offset.as_mut().context("Parameters were initialized with this field.")?;

		if backwards {
			*offset += chunk_size as i32;
		} else {
			*offset -= chunk_size as i32;
		}
	}
}
