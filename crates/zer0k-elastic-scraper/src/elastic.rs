use {
	chrono::{DateTime, Utc},
	color_eyre::{
		eyre::{eyre, Context},
		Result,
	},
	elasticsearch::{
		http::{request::JsonBody, transport::Transport},
		Elasticsearch, Scroll, ScrollParts, SearchParts,
	},
	gokz_rs::types::{Mode, SteamID},
	serde::{Deserialize, Deserializer, Serialize},
	serde_json::{json, Value as JsonValue},
	tracing::warn,
};

pub const SCROLL_DURATION: &str = "4m";

pub type ScrollId = String;

/// The initial response from Elastic
#[derive(Debug, Deserialize)]
pub struct Response {
	#[serde(rename = "_scroll_id")]
	pub scroll_id: ScrollId,

	#[serde(flatten)]
	pub body: JsonValue,
}

/// A single KZ record from Elastic
#[derive(Debug, Clone, Serialize)]
pub struct Record {
	pub id: u32,
	pub map_name: String,
	pub stage: u8,
	pub mode: Mode,
	pub player_name: String,

	#[serde(rename = "steamid64")]
	pub steam_id: SteamID,

	pub teleports: u32,
	pub time: f64,
	pub server_name: String,

	#[serde(
		serialize_with = "crate::utils::ser_date",
		deserialize_with = "crate::utils::deser_date"
	)]
	pub created_on: DateTime<Utc>,
}

impl<'de> Deserialize<'de> for Record {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
		#[derive(Debug, Deserialize)]
		struct _Record {
			pub id: u32,
			pub map_name: Option<String>,
			pub stage: u8,
			pub mode: Mode,
			pub player_name: Option<String>,

			#[serde(rename = "steamid64")]
			pub steam_id: SteamID,

			pub teleports: u32,
			pub time: f64,
			pub server_name: Option<String>,

			#[serde(
				serialize_with = "crate::utils::ser_date",
				deserialize_with = "crate::utils::deser_date"
			)]
			pub created_on: DateTime<Utc>,
		}

		let _Record {
			id,
			map_name,
			stage,
			mode,
			player_name,
			steam_id,
			teleports,
			time,
			server_name,
			created_on,
		} = _Record::deserialize(deserializer)?;

		Ok(Self {
			id,
			map_name: map_name.unwrap_or_else(|| String::from("unknown map")),
			stage,
			mode,
			player_name: player_name.unwrap_or_else(|| String::from("unknown player")),
			steam_id,
			teleports,
			time,
			server_name: server_name.unwrap_or_else(|| String::from("unknown server")),
			created_on,
		})
	}
}

#[derive(Debug)]
pub struct Payload {
	pub scroll_id: ScrollId,
	pub records: Vec<Record>,

	/// Unsuccessfully parsed [`Record`]s
	pub failures: Vec<JsonValue>,
}

#[tracing::instrument(level = "DEBUG", err(Debug))]
pub async fn fetch_initial(
	chunk_size: usize,
	search_parts: SearchParts<'_>,
	client: &Elasticsearch,
) -> Result<Payload> {
	let response = client
		.search(search_parts)
		.size(chunk_size as i64)
		.scroll(SCROLL_DURATION)
		.body(json!({
			"query": {
				"match_all": {}
			}
		}))
		.send()
		.await
		.context("Fetch request to elastic failed.")?
		.json::<Response>()
		.await
		.context("Failed to parse elastic response.")?;

	process_response(response)
}

#[tracing::instrument(level = "DEBUG", err(Debug))]
pub async fn fetch(scroll_id: &str, transport: &Transport) -> Result<Payload> {
	let response = Scroll::<JsonBody<JsonValue>>::new(transport, ScrollParts::ScrollId(scroll_id))
		.body(json!({
			"scroll": SCROLL_DURATION,
			"scroll_id": scroll_id
		}))
		.send()
		.await
		.context("Fetch request to elastic failed.")?
		.json::<Response>()
		.await
		.context("Failed to parse elastic response.")?;

	process_response(response)
}

#[tracing::instrument(level = "DEBUG", err(Debug))]
pub fn process_response(response: Response) -> Result<Payload> {
	let scroll_id = response.scroll_id;

	let results = response.body["hits"]["hits"]
		.as_array()
		.ok_or(eyre!("Elastic response has incorrect shape. Expected array."))?;

	let mut records = Vec::new();
	let mut failures = Vec::new();

	for record in results {
		let record = &record["_source"];

		match serde_json::from_value(record.to_owned()) {
			Ok(record) => {
				records.push(record);
			}

			Err(err) => {
				warn!(?err, ?record, "Found a malformed record.");
				failures.push(record.to_owned());
			}
		}
	}

	Ok(Payload {
		scroll_id,
		records,
		failures,
	})
}
