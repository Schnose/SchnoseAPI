use {
	color_eyre::{eyre::Context, Result},
	sqlx::{FromRow, MySqlPool, PgPool, QueryBuilder},
	tracing::{info, trace},
};

/// +-----------+--------------+------+-----+---------+-------+
/// | Field     | Type         | Null | Key | Default | Extra |
/// +-----------+--------------+------+-----+---------+-------+
/// | id        | int unsigned | NO   | PRI | NULL    |       |
/// | name      | varchar(255) | NO   | MUL | unknown |       |
/// | is_banned | tinyint(1)   | NO   |     | 0       |       |
/// +-----------+--------------+------+-----+---------+-------+
#[derive(Debug, Clone, FromRow)]
pub struct Player {
	pub id: u32,
	pub name: String,
	pub is_banned: bool,
}

#[tracing::instrument(level = "INFO", skip(pool), err(Debug))]
pub async fn select_players(offset: isize, limit: usize, pool: &MySqlPool) -> Result<Vec<Player>> {
	let mut query = QueryBuilder::new("SELECT * FROM players LIMIT ");
	query.push_bind(limit as u64).push(" OFFSET ").push_bind(offset as i64);
	let players = query.build_query_as().fetch_all(pool).await?;

	info!(amount = %players.len(), "Fetched players.");

	Ok(players)
}

#[tracing::instrument(
	level = "INFO",
	skip(players, pool),
	fields(amount = %players.len()),
	err(Debug)
)]
pub async fn insert_players(players: Vec<Player>, pool: &PgPool) -> Result<usize> {
	let mut query = QueryBuilder::new(
		r#"
		INSERT INTO players
			(id, name, is_banned)
		"#,
	);

	let processed = players.len();

	for players in players.chunks(1000) {
		trace!(size = %players.len(), "processing chunk of players");

		let mut transaction = pool.begin().await.context("Failed to start SQL transaction.")?;

		query.push_values(
			players,
			|mut query,
			 Player {
			     id,
			     name,
			     is_banned,
			 }| {
				query.push_bind(*id as i32).push_bind(name).push_bind(is_banned);
			},
		);

		trace!("building query");
		query.build().execute(&mut transaction).await.context("Failed to execute query.")?;

		trace!("committing query");
		transaction.commit().await.context("Failed to commit SQL transaction.")?;

		query.reset();
	}

	Ok(processed)
}
