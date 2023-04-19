use {
	crate::elastic::ElasticRecord,
	chrono::{DateTime, NaiveDateTime, Utc},
	gokz_rs::{Mode, SteamID},
	serde::{de, Deserialize, Deserializer, Serialize, Serializer},
};

pub(crate) fn ser_date<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	date.format("%Y-%m-%dT%H:%M:%S")
		.to_string()
		.serialize(serializer)
}

pub fn deser_date<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
	D: Deserializer<'de>,
{
	let date = String::deserialize(deserializer)?;
	NaiveDateTime::parse_from_str(&date, "%Y-%m-%dT%H:%M:%S")
		.map(|datetime| DateTime::<Utc>::from_utc(datetime, Utc))
		.map_err(|err| de::Error::custom(err.to_string()))
}

pub fn treat_error_as_none<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
	T: Deserialize<'de>,
	D: Deserializer<'de>,
{
	Ok(T::deserialize(deserializer).ok())
}

impl<'de> Deserialize<'de> for crate::elastic::ElasticRecord {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		struct HasServer {
			id: u32,
			map_name: String,
			stage: u8,
			mode: Mode,
			player_name: String,
			#[serde(rename = "steamid64")]
			steam_id: SteamID,
			teleports: u32,
			time: f64,
			server: String,
			tickrate: u8,
			#[serde(serialize_with = "crate::serde::ser_date")]
			#[serde(deserialize_with = "crate::serde::deser_date")]
			created_on: DateTime<Utc>,
		}

		#[derive(Deserialize)]
		struct HasServerName {
			id: u32,
			map_name: String,
			stage: u8,
			mode: Mode,
			player_name: String,
			#[serde(rename = "steamid64")]
			steam_id: SteamID,
			teleports: u32,
			time: f64,
			server_name: String,
			tickrate: u8,
			#[serde(serialize_with = "crate::serde::ser_date")]
			#[serde(deserialize_with = "crate::serde::deser_date")]
			created_on: DateTime<Utc>,
		}

		let value = serde_json::Value::deserialize(deserializer)?;

		match serde_json::from_value::<HasServer>(value.clone()) {
			Ok(record) => Ok(ElasticRecord {
				id: record.id,
				map_name: record.map_name,
				stage: record.stage,
				mode: record.mode,
				player_name: record.player_name,
				steam_id: record.steam_id,
				teleports: record.teleports,
				time: record.time,
				server_name: record.server,
				tickrate: record.tickrate,
				created_on: record.created_on,
			}),
			Err(_) => serde_json::from_value::<HasServerName>(value)
				.map(|record| ElasticRecord {
					id: record.id,
					map_name: record.map_name,
					stage: record.stage,
					mode: record.mode,
					player_name: record.player_name,
					steam_id: record.steam_id,
					teleports: record.teleports,
					time: record.time,
					server_name: record.server_name,
					tickrate: record.tickrate,
					created_on: record.created_on,
				})
				.map_err(|err| de::Error::custom(err.to_string())),
		}
	}
}
