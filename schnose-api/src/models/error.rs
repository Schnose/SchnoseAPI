use {
	axum::{http::StatusCode, response::IntoResponse, Json},
	thiserror::Error,
	tracing::error,
	utoipa::ToSchema,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Error, ToSchema)]
pub enum Error {
	/// Any error that only occurrs in 1 place and is not worth having its own variant.
	#[error("{0}")]
	Custom(String),

	/// An error that is not supposed to happen and which the user should not find out about.
	#[error("An unexpected error occurred.")]
	Hidden(String),

	/// A database query yielded 0 rows.
	#[error("No data found.")]
	NoContent,

	#[error("The date values you submitted do not make sense.")]
	InvalidDates,
}

impl IntoResponse for Error {
	#[tracing::instrument(level = "TRACE")]
	fn into_response(self) -> axum::response::Response {
		let (code, message) = match self {
			Self::Custom(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
			Self::Hidden(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
			Self::NoContent => (StatusCode::NO_CONTENT, self.to_string()),
			Self::InvalidDates => (StatusCode::BAD_REQUEST, self.to_string()),
		};

		(code, Json(message)).into_response()
	}
}

impl From<color_eyre::Report> for Error {
	#[allow(unused_braces)]
	#[tracing::instrument(level = "ERROR", fields(source = ?error.source()), ret)]
	fn from(error: color_eyre::Report) -> Self { Self::Custom(error.to_string()) }
}

impl From<std::num::TryFromIntError> for Error {
	#[allow(unused_braces)]
	#[tracing::instrument(level = "TRACE", ret)]
	fn from(error: std::num::TryFromIntError) -> Self { Self::Hidden(error.to_string()) }
}

impl From<gokz_rs::error::Error> for Error {
	#[tracing::instrument(level = "TRACE", ret)]
	fn from(error: gokz_rs::error::Error) -> Self {
		match error {
			gokz_rs::prelude::Error::Custom(error)
			| gokz_rs::prelude::Error::InvalidUrl(error)
			| gokz_rs::prelude::Error::Reqwest(error) => Self::Hidden(error),

			gokz_rs::prelude::Error::Http {
				..
			}
			| gokz_rs::prelude::Error::EmptyResponse => Self::Custom(error.to_string()),

			_ => todo!(),
		}
	}
}

impl From<sqlx::Error> for Error {
	fn from(error: sqlx::Error) -> Self {
		#[allow(clippy::match_single_binding)] // TODO: remove
		match error {
			// sqlx::Error::Configuration(_) => todo!(),
			// sqlx::Error::Database(_) => todo!(),
			// sqlx::Error::Io(_) => todo!(),
			// sqlx::Error::Tls(_) => todo!(),
			// sqlx::Error::Protocol(_) => todo!(),
			// sqlx::Error::RowNotFound => todo!(),
			// sqlx::Error::TypeNotFound {
			// 	type_name,
			// } => todo!(),
			// sqlx::Error::ColumnIndexOutOfBounds {
			// 	index,
			// 	len,
			// } => todo!(),
			// sqlx::Error::ColumnNotFound(_) => todo!(),
			// sqlx::Error::ColumnDecode {
			// 	index,
			// 	source,
			// } => todo!(),
			// sqlx::Error::Decode(_) => todo!(),
			// sqlx::Error::PoolTimedOut => todo!(),
			// sqlx::Error::PoolClosed => todo!(),
			// sqlx::Error::WorkerCrashed => todo!(),
			// sqlx::Error::Migrate(_) => todo!(),
			_ => Self::Hidden(String::from("Database error.")),
		}
	}
}
