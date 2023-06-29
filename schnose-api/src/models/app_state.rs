pub struct AppState {
	pub pool: sqlx::PgPool,
}

impl AppState {
	pub fn db(&self) -> &sqlx::PgPool { &self.pool }
}
