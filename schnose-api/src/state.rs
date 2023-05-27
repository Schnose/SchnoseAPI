use {
	crate::routes,
	axum::{routing::get, Router, Server, ServiceExt},
	shuttle_runtime::async_trait,
	sqlx::{mysql::MySqlPoolOptions, MySql, Pool},
	std::{net::SocketAddr, sync::Arc},
	tower::Layer,
	tower_http::normalize_path::NormalizePathLayer,
	tracing::{error, info},
};

pub type ShuttleResult = Result<APIState, shuttle_service::Error>;

#[async_trait]
impl shuttle_service::Service for APIState {
	async fn bind(self, addr: SocketAddr) -> Result<(), shuttle_service::Error> {
		let server = Server::bind(&addr);

		info!("Listening on {addr}.");

		let router = Router::new()
			.route("/health", get(|| async { "(͡ ͡° ͜ つ ͡͡°)" }))
			.route("/api/modes", get(routes::modes::root::get))
			.route("/api/modes/:ident", get(routes::modes::ident::get))
			.route("/api/players", get(routes::players::root::get))
			.route("/api/players/:ident", get(routes::players::ident::get))
			.route("/api/maps", get(routes::maps::root::get))
			.route("/api/maps/:ident", get(routes::maps::ident::get))
			.route("/api/servers", get(routes::servers::root::get))
			.route("/api/servers/:ident", get(routes::servers::ident::get))
			.route("/api/records", get(routes::records::root::get))
			.with_state(self);

		let router = NormalizePathLayer::trim_trailing_slash().layer(router);

		tokio::select! {
			res = server.serve(router.into_make_service()) => {
				error!("{res:?}");
			}
		};

		Ok(())
	}
}

#[derive(Debug, Clone)]
pub struct APIState {
	pub database_connection: Arc<Pool<MySql>>,
}

impl APIState {
	#[tracing::instrument(skip(connection_string))]
	pub async fn new(connection_string: &str) -> Self {
		let database_connection = MySqlPoolOptions::new()
			.connect(connection_string)
			.await
			.expect("Failed to establish database connection.");

		Self {
			database_connection: Arc::new(database_connection),
		}
	}

	pub fn db(&self) -> &Pool<MySql> {
		&self.database_connection
	}
}
