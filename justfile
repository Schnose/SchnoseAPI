# Show all available commands
help:
	@just --list

# Generate git hooks
hooks:
	cd hooks && ./create.sh

# Clippy
check:
	cargo clippy --workspace --all-features -- -D warnings

# Formatting
fmt:
	cargo +nightly fmt --all

# Tests
test:
	RUST_BACKTRACE=0 cargo test --workspace --all-features

# Generate documentation
doc:
	cargo doc --all-features --document-private-items

# Prepare SQL migrations
prepare:
	cd ./schnose-api && cargo sqlx prepare

# Run SQL migrations
migrate:
	cargo sqlx migrate run

# Revert SQL migrations
revert:
	cargo sqlx migrate revert

# Start the local database container
db-up:
	docker compose up schnose-postgres -d

# Stop the local database container
db-down:
	docker compose stop schnose-postgres

# Restart the local database container
db-restart:
	@just db-down
	@just db-up

# Connect to the local database container
db-connect:
	PGPASSWORD=postgres psql -U postgres -h 127.0.0.1 -p 9001 -d schnose-api-dev

# Start the API container
api-up:
	docker compose up schnose-api

# Start the API container without attaching to it
api-upd:
	docker compose up schnose-api -d

# Stop the API container
api-down:
	docker compose stop schnose-api

# Rebuild and start the API container
api-build:
	docker compose up schnose-api --build

# Compile and run the API in dev mode locally
dev:
	RUST_LOG=ERROR,schnose_api=DEBUG cargo run \
		--bin schnose-api \
		-- \
			--config ./configs/api.toml \
			--port 9002

# Compile and run the API in release mode locally
dev-release:
	RUST_LOG=ERROR,schnose_api=DEBUG cargo run \
		--release \
		--bin schnose-api \
		-- \
			--config ./configs/api.toml \
			--port 9002

# Fetch records from zer0k's elastic instance
scrape-elastic:
	cargo run \
		--release \
		--bin zer0k-elastic-scraper \
		-- \
			-c crates/zer0k-elastic-scraper/config.toml \
			-o ./data/elastic-scraper

# Fetch maps from the GlobalAPI
scrape-maps:
	cargo run \
		--release \
		--bin global-api-scraper \
		-- \
			-c configs/data-wrangler.toml \
			maps

# Fetch maps from the GlobalAPI
scrape-servers:
	cargo run \
		--release \
		--bin global-api-scraper \
		-- \
			-c configs/data-wrangler.toml \
			servers

