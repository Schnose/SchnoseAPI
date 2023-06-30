use axum::{http, Json};

#[rustfmt::skip]
#[utoipa::path(
	get,
	tag = "Health",
	path = "/api/health",
	responses(
		(status = 200, description = "Healthcheck for the API", body = &'static str),
	),
)]
#[tracing::instrument(level = "DEBUG")]
pub async fn root(method: http::Method) -> Json<&'static str> {
	Json("(͡ ͡° ͜ つ ͡͡°)")
}
