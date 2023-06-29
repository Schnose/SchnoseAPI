use {crate::routes, utoipa::OpenApi, utoipa_swagger_ui::SwaggerUi};

#[rustfmt::skip]
#[derive(OpenApi)]
#[openapi(
	paths(
		routes::health::root,
		routes::players::root,
	),

	components(
		schemas(
			crate::Error,
			crate::database::players::Player,
		),
	),
)]
pub struct SchnoseAPI;

impl SchnoseAPI {
	#[tracing::instrument(level = "TRACE")]
	pub fn swagger() -> SwaggerUi {
		SwaggerUi::new("/docs/swagger").url("/docs/spec.json", Self::openapi())
	}

	#[tracing::instrument(level = "TRACE")]
	pub fn routes() -> impl Iterator<Item = String> {
		SchnoseAPI::openapi().paths.paths.into_iter().map(|path| path.0)
	}
}