pub mod database;

pub mod models;
pub use models::{
	error::{Error, Result},
	spec::SchnoseAPI,
};

pub mod routes;

/// Convenience enum for building queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
	Where,
	And,
}

impl std::fmt::Display for Filter {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			Filter::Where => " WHERE ",
			Filter::And => " AND ",
		})
	}
}

impl Filter {
	/// Changes `self` to `Self::And`.
	pub fn and(&mut self) { *self = Self::And; }
}
