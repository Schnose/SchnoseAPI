pub mod database;

pub mod models;
pub use models::{
	error::{Error, Result},
	spec::SchnoseAPI,
};

pub mod routes;
