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
		--release \
		--bin zer0k-elastic-scraper \
		-- \
			-c crates/zer0k-elastic-scraper/config.toml \
			-o ./data/elastic-scraper

# Run SQL migrations
migrate:
	cargo sqlx migrate run

# Revert SQL migrations
revert:
	cargo sqlx migrate revert

# Spin up the docker containers
up:
	docker-compose up -d

# Run the PostgreSQL container
db-start:
	docker-compose run schnose-postgres

# Stop the PostgreSQL container
db-stop:
	docker-compose stop schnose-postgres

# Connect to the PostgreSQL container
db-connect:
	PGPASSWORD=postgres psql -U postgres -h 127.0.0.1 -p 9001 -d schnose-api-dev

# Delete the docker volumes
clean-volumes:
	docker volume rm schnose-postgres
	docker volume rm schnose-docker-target

# Run the API in dev mode
dev:
	RUST_LOG=ERROR,schnose_api=DEBUG cargo run \
		--bin schnose-api \
		-- \
			--config ./api/config.toml \
			--port 9002

# Run the API in release mode
prod:
	RUST_LOG=ERROR,schnose_api=TRACE cargo run \
		--relase \
		--bin schnose-api \
		-- \
			--config ./api/config.toml \
			--port 9002 \
			--public

