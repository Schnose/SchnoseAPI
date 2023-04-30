use std::time::Duration;

pub mod fetch_players;
pub mod fetch_records;

pub const FETCH_DELAY: Duration = Duration::from_millis(727);

#[derive(sqlx::FromRow)]
pub struct RecordID(pub u32);
