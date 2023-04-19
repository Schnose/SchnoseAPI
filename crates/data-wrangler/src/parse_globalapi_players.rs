use {
	crate::Args,
	color_eyre::{eyre::Context, Result},
	gokz_rs::global_api::Player,
	schnosedb::models::PlayerRow,
	sqlx::{MySql, Pool},
};

pub fn parse(players: Vec<Player>, database_connection: &Pool<MySql>, args: &Args) -> Result<()> {
	let players = players
		.into_iter()
		.map(|player| PlayerRow {
			id: player.steam_id.as_id32(),
			name: player.name,
			is_banned: player.is_banned,
		})
		.collect::<Vec<_>>();

	let bytes = serde_json::to_vec(&players).context("Failed to serialize records.")?;
	std::fs::write(&args.output_path, &bytes).context("Failed to write JSON to disk.")
}
