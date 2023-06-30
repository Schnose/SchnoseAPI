use {
	crate::{database, routes},
	utoipa::OpenApi,
	utoipa_swagger_ui::SwaggerUi,
};

#[rustfmt::skip]
#[derive(OpenApi)]
#[openapi(
	info(
		title = "SchnoseAPI",
		version = "1",
		contact(
			name = "AlphaKeks",
			email = "alphakeks@dawn.sh",
		),
		license(
			name = "GPL-3.0",
			url = "https://www.gnu.org/licenses/gpl-3.0",
		),
	),

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
		routes::filters::map::map,
		routes::records::root,
		routes::records::id::id,
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
