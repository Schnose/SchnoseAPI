use {
	crate::Args,
	color_eyre::{eyre::Context, Result},
	gokz_rs::global_api::Server,
	schnosedb::models::ServerRow,
};

const ZPAMM: u32 = 17690692;

pub fn parse(servers: Vec<Server>, args: &Args) -> Result<()> {
	let servers = servers
		.into_iter()
		.map(|server| ServerRow {
			id: server.id,
			name: server.name,
			owned_by: server.owner_steamid.as_id32(),
			approved_by: ZPAMM,
		})
		.collect::<Vec<_>>();

	let bytes = serde_json::to_vec(&servers).context("Failed to serialize records.")?;
	std::fs::write(&args.output_path, bytes).context("Failed to write JSON to disk.")
}
