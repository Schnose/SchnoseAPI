use {
	axum::{extract::Json, http::StatusCode, response::IntoResponse},
	schnose_api::error::Error,
	serde::{Deserialize, Serialize},
};

pub type Response<T> = Result<ResponseBody<T>, Error>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseBody<T>(pub T);

impl<T> From<T> for ResponseBody<T> {
	fn from(json: T) -> Self {
		Self(json)
	}
}

impl<T: Serialize> IntoResponse for ResponseBody<T> {
	fn into_response(self) -> axum::response::Response {
		(StatusCode::OK, Json(self.0)).into_response()
	}
}
