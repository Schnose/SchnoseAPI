use serde::{de, Deserialize, Deserializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum Bool {
	Int(u8),
	Bool(bool),
}

impl TryFrom<Bool> for bool {
	type Error = u8;

	fn try_from(value: Bool) -> Result<Self, Self::Error> {
		Ok(match value {
			Bool::Int(int) => match int {
				0 => false,
				1 => true,
				n => return Err(n),
			},
			Bool::Bool(bool) => bool,
		})
	}
}

pub fn deser_sql_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
	D: Deserializer<'de>,
{
	Bool::deserialize(deserializer)?
		.try_into()
		.map_err(|n| {
			de::Error::invalid_value(de::Unexpected::Unsigned(n as u64), &"bool must be 0 or 1")
		})
}
