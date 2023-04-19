use {
	serde::{de, Deserialize, Deserializer, Serialize, Serializer},
	sqlx::types::chrono::{DateTime, NaiveDateTime, Utc},
};

pub fn serialize_datetime<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	date.format("%Y-%m-%dT%H:%M:%S")
		.to_string()
		.serialize(serializer)
}

pub fn deserialize_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
	D: Deserializer<'de>,
{
	let date = String::deserialize(deserializer)?;
	NaiveDateTime::parse_from_str(&date, "%Y-%m-%dT%H:%M:%S")
		.map(|datetime| DateTime::<Utc>::from_utc(datetime, Utc))
		.map_err(|err| de::Error::custom(err.to_string()))
}
