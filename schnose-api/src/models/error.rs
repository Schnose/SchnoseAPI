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
}

impl IntoResponse for Error {
	#[tracing::instrument(level = "TRACE")]
	fn into_response(self) -> axum::response::Response {
		let (code, message) = match self {
			Self::Custom(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
			Self::Hidden(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
		};

		(code, Json(message)).into_response()
	}
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
				// code,
				// message,
				..
			}
			| gokz_rs::prelude::Error::EmptyResponse => Self::Custom(error.to_string()),

			_ => todo!(),
		}
	}
}
