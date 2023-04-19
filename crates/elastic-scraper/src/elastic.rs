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
	gokz_rs::{Mode, SteamID},
	serde::{Deserialize, Serialize},
	serde_json::json,
};

pub const SCROLL_DURATION: &str = "4m";

#[derive(Debug, Deserialize)]
pub struct ElasticResponse {
	#[serde(rename = "_scroll_id")]
	scroll_id: String,
	#[serde(flatten)]
	body: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ElasticRecord {
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
	pub tickrate: u8,
	#[serde(serialize_with = "crate::serde::ser_date")]
	#[serde(deserialize_with = "crate::serde::deser_date")]
	pub created_on: DateTime<Utc>,
}

pub type ScrollID = String;

pub async fn fetch_initial(
	chunk_size: usize,
	search_parts: SearchParts<'_>,
	elastic_client: &Elasticsearch,
) -> Result<(ScrollID, (Vec<ElasticRecord>, Vec<serde_json::Value>))> {
	let response = elastic_client
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
		.context("Failed to fetch records from Elastic.")?
		.json::<ElasticResponse>()
		.await
		.context("Failed to deserialize Elastic response.")?;

	let scroll_id = response.scroll_id;
	let hits = response.body["hits"]["hits"]
		.as_array()
		.ok_or(eyre!("Response body has incorrect shape. Expected array."))?;

	let mut records = Vec::new();
	let mut malformed_records = Vec::new();

	for hit in hits {
		let record = &hit["_source"];
		let record: ElasticRecord = match serde_json::from_value(record.to_owned()) {
			Ok(record) => record,
			Err(_) => {
				malformed_records.push(record.to_owned());
				continue;
			}
		};

		records.push(record);
	}

	Ok((scroll_id, (records, malformed_records)))
}

pub async fn scroll(
	transport: &Transport,
	scroll_id: &str,
) -> Result<(ScrollID, (Vec<ElasticRecord>, Vec<serde_json::Value>))> {
	let response = Scroll::<'_, '_, JsonBody<serde_json::Value>>::new(
		transport,
		ScrollParts::ScrollId(scroll_id),
	)
	.body(json!({
		"scroll": SCROLL_DURATION,
		"scroll_id": scroll_id
	}))
	.send()
	.await
	.context("Failed to send scroll request")?
	.json::<ElasticResponse>()
	.await?;

	let scroll_id = response.scroll_id;
	let hits = response.body["hits"]["hits"]
		.as_array()
		.ok_or(eyre!("Response body has incorrect shape. Expected array."))?;

	let mut records = Vec::new();
	let mut malformed_records = Vec::new();

	for hit in hits {
		let record = hit["_source"].clone();
		let record: ElasticRecord = match serde_json::from_value(record.to_owned()) {
			Ok(record) => record,
			Err(_) => {
				malformed_records.push(record.to_owned());
				continue;
			}
		};

		records.push(record);
	}

	Ok((scroll_id, (records, malformed_records)))
}
