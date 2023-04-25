use {
	axum::{extract::Json, http::StatusCode, response::IntoResponse},
	thiserror::Error,
	tracing::{error, warn},
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum Error {
	#[error("{0}")]
	Custom(&'static str),

	#[error("Database error")]
	Database,

	#[error("No content")]
	NoContent,
}

impl IntoResponse for Error {
	fn into_response(self) -> axum::response::Response {
		match self {
			Error::Custom(msg) => (StatusCode::INTERNAL_SERVER_ERROR, Json(msg.to_owned())),
			err @ Error::Database => (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())),
			err @ Error::NoContent => (StatusCode::NO_CONTENT, Json(err.to_string())),
		}
		.into_response()
	}
}

impl From<&'static str> for Error {
	fn from(value: &'static str) -> Self {
		Self::Custom(value)
	}
}

#[allow(clippy::cognitive_complexity)]
impl From<sqlx::Error> for Error {
	fn from(value: sqlx::Error) -> Self {
		match &value {
			sqlx::Error::Database(db_err) => {
				error!("Database error! {db_err:#?}");
				Self::Database
			}
			sqlx::Error::Io(db_err) => {
				error!("Database I/O error! {db_err:#?}");
				Self::Database
			}
			sqlx::Error::RowNotFound => {
				warn!("{value:?}");
				Self::NoContent
			}
			err => {
				error!("{err:#?}");
				Self::Database
			}
		}
	}
}

#[allow(unused)]
macro_rules! yeet {
	($error:expr) => {
		return Err($error.into());
	};
}

#[allow(unused)]
pub(crate) use yeet;
