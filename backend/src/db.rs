use diesel::{
    r2d2::{ConnectionManager, Pool, PoolError, PooledConnection},
    sqlite::SqliteConnection,
    Connection,
};
use std::env;

pub(crate) type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;
pub(crate) type SqlitePooledConnection = PooledConnection<ConnectionManager<SqliteConnection>>;

pub(crate) fn database_url() -> String {
    let _ = dotenvy::dotenv();
    env::var("DATABASE_URL").unwrap_or_else(|_| "./minions.sqlite3".to_owned())
}

pub(crate) fn establish_connection(
    database_url: &str,
) -> Result<SqliteConnection, diesel::ConnectionError> {
    SqliteConnection::establish(database_url)
}

pub(crate) fn create_pool(database_url: &str) -> Result<SqlitePool, PoolError> {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    Pool::builder().build(manager)
}
