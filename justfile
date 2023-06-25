# Show all available commands
help:
	@just --list

# Generate git hooks (this will override existing hooks!)
hooks:
	./hooks/create.sh

# Analyze the codebase with clippy™️
check:
	cargo clippy --workspace --all-features -- -D warnings

# Format the codebase with nightly rustfmt
fmt:
	cargo +nightly fmt --all

# Run tests
test:
	RUST_BACKTRACE=0 cargo test --workspace --all-features

# Generate documentation
doc:
	cargo doc --all-features --document-private-items

# Fetch records from zer0k's elastic instance
scrape-elastic:
	cargo run \
		--bin zer0k-elastic-scraper \
		--release \
		-- \
			-c crates/zer0k-elastic-scraper/config.toml \
			-o ./data/elastic-scraper

