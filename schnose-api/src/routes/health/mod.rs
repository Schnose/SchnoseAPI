use axum::{http, Json};

#[allow(unused_braces)]
#[utoipa::path(
	get,
	path = "/health",
	responses(
		(status = 200, description = "Healthcheck for the API", body = &'static str),
	),
)]
#[tracing::instrument(level = "TRACE")]
pub async fn root(method: http::Method) -> Json<&'static str> { Json("(͡ ͡° ͜ つ ͡͡°)") }
