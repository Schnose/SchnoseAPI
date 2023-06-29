use {
	crate::{database, routes},
	utoipa::OpenApi,
	utoipa_swagger_ui::SwaggerUi,
};

#[rustfmt::skip]
#[derive(OpenApi)]
#[openapi(
	paths(
		routes::health::root,
		routes::players::root,
		routes::players::ident::ident,
		routes::modes::root,
		routes::modes::ident::ident,
		routes::maps::root,
		routes::maps::ident::ident,
		routes::servers::root,
		routes::servers::ident::ident,
	),

	components(
		schemas(
			crate::Error,
			database::Course,
			database::Filter,
			database::Mapper,
			database::MapModel,
			database::Mode,
			database::Player,
			database::Record,
			database::Server,
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
