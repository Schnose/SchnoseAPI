use {crate::routes, utoipa::OpenApi, utoipa_swagger_ui::SwaggerUi};

#[rustfmt::skip]
#[derive(OpenApi)]
#[openapi(
	paths(
		routes::health::root,
	),

	components(
		schemas(

		),
	),
)]
pub struct SchnoseAPI;

impl SchnoseAPI {
	pub fn swagger() -> SwaggerUi {
		SwaggerUi::new("/docs/swagger").url("/docs/spec.json", Self::openapi())
	}

	pub fn routes() -> impl Iterator<Item = String> {
		SchnoseAPI::openapi().paths.paths.into_iter().map(|path| path.0)
	}
}