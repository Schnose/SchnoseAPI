use {
	axum::{routing::get, Router, Server},
	color_eyre::{eyre::Context, Result},
	std::net::SocketAddr,
};

#[tokio::main]
async fn main() -> Result<()> {
	let router = Router::new().route("/", get(|| async { "(͡ ͡° ͜ つ ͡͡°)" }));

	let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
	let server = Server::bind(&addr);

	server
		.serve(router.into_make_service())
		.await
		.context("Failed to run server.")?;

	Ok(())
}
