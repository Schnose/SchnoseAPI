use {
	chrono::{DateTime, NaiveDateTime, Utc},
	color_eyre::{eyre::Context, Result},
	serde::{Deserialize, Deserializer, Serialize, Serializer},
	std::path::Path,
	tracing::trace,
};

pub fn ser_date<S: Serializer>(
	date_time: &DateTime<Utc>,
	serializer: S,
) -> Result<S::Ok, S::Error> {
	date_time.format("%Y-%m-%dT%H:%M:%S").to_string().serialize(serializer)
}

pub fn deser_date<'de, D: Deserializer<'de>>(deserializer: D) -> Result<DateTime<Utc>, D::Error> {
	let date = String::deserialize(deserializer)?;
	NaiveDateTime::parse_from_str(&date, "%Y-%m-%dT%H:%M:%S")
		.map(|datetime| DateTime::<Utc>::from_utc(datetime, Utc))
		.map_err(|err| serde::de::Error::custom(err.to_string()))
}

#[tracing::instrument(level = "TRACE", skip(data), err(Debug))]
pub async fn save_json<S: Serialize>(data: &S, output_path: &Path) -> Result<()> {
	let json = serde_json::to_string(data).context("Failed to serialize data.")?;
	tokio::fs::write(output_path, &json).await.context("Failed to write json to file.")?;
	trace!(bytes = %json.len(), output_path = %output_path.display(), "Wrote data to disk.");

	Ok(())
}
