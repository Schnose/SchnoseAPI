use {
	crate::Args,
	color_eyre::{eyre::Context, Result},
	gokz_rs::SteamID,
	schnosedb::models::ServerRow,
	serde::{Deserialize, Deserializer},
};

const ZPAMM: u32 = 17690692;

#[derive(Debug, Deserialize)]
pub struct Server {
	pub id: u16,
	#[serde(deserialize_with = "deserialize_unknown")]
	pub name: String,
	#[serde(deserialize_with = "deserialize_steam_id")]
	pub owner_steamid: u32,
	#[serde(deserialize_with = "deserialize_unknown")]
	pub ip: String,
	pub port: u16,
}

fn deserialize_unknown<'de, D: Deserializer<'de>>(
	deserializer: D,
) -> std::result::Result<String, D::Error> {
	Ok(match serde_json::Value::deserialize(deserializer)? {
		serde_json::Value::Null => String::from("unknown"),
		serde_json::Value::String(name) if name.is_empty() => String::from("unknown"),
		serde_json::Value::String(name) => name,
		_ => String::from("unknown"),
	})
}

fn deserialize_steam_id<'de, D: Deserializer<'de>>(
	deserializer: D,
) -> std::result::Result<u32, D::Error> {
	let value = serde_json::Value::deserialize(deserializer)?;

	if let serde_json::Value::Null = value {
		return Ok(0);
	}

	if let serde_json::Value::String(s) = value {
		if let Ok(steam_id) = SteamID::new(s) {
			return Ok(steam_id.as_id32());
		}
	}

	Ok(0)
}

pub fn parse(servers: Vec<Server>, args: &Args) -> Result<()> {
	let servers = servers
		.into_iter()
		.map(|server| ServerRow {
			id: server.id,
			name: server.name,
			owned_by: server.owner_steamid,
			approved_by: ZPAMM,
		})
		.collect::<Vec<_>>();

	let bytes = serde_json::to_vec(&servers).context("Failed to serialize records.")?;
	std::fs::write(&args.output_path, bytes).context("Failed to write JSON to disk.")
}
