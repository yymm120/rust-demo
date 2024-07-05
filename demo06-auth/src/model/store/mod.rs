// region:    --- Modules

mod error;

pub use self::error::{Error, Result};

use crate::config;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use tracing::info;

// endregion: --- Modules

pub type Db = Pool<Postgres>;

pub async fn new_db_pool() -> Result<Db> {
	info!("--> PWD_KEY is {:?}", &config().PWD_KEY);
	PgPoolOptions::new()
		.max_connections(5)
		.connect(&config().DB_URL)
		.await
		.map_err(|ex| Error::FailToCreatePool(ex.to_string()))
}