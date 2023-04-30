//!

#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms)]
#![warn(clippy::style, clippy::cognitive_complexity, clippy::complexity)]
#![deny(clippy::perf, clippy::correctness)]

use {shuttle_secrets::SecretStore, state::APIState, state::ShuttleResult};

mod response;
mod routes;
mod state;

#[shuttle_runtime::main]
async fn schnoseapi(#[shuttle_secrets::Secrets] secrets: SecretStore) -> ShuttleResult {
	let connection_string = secrets
		.get("CONNECTION_STRING")
		.expect("Missing `CONNECTION_STRING` secret.");

	let state = APIState::new(&connection_string).await;

	Ok(state)
}
