use {serde::Serialize, std::path::PathBuf, tracing::trace};

pub async fn save_as_json(data: impl Serialize, target_dir: PathBuf) {
	let json = serde_json::to_vec(&data).expect("Failed to serialize data.");
	tokio::fs::write(&target_dir, &json)
		.await
		.unwrap_or_else(|_| panic!("Failed to write JSON data to `{}`.", target_dir.display()));
	trace!("Wrote {} bytes to `{}`.", json.len(), target_dir.display());
}
