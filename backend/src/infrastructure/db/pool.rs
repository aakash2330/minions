use diesel::{
    r2d2::{ConnectionManager, Pool, PoolError},
    sqlite::SqliteConnection,
    Connection,
};
use std::{env, error::Error, sync::OnceLock};

pub(crate) type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;
pub(crate) type DbError = Box<dyn Error + Send + Sync>;

static SHARED_POOL: OnceLock<SqlitePool> = OnceLock::new();

pub(crate) fn database_url() -> String {
    let _ = dotenvy::dotenv();
    env::var("DATABASE_URL").unwrap_or_else(|_| "./sessions.sqlite3".to_owned())
}

#[allow(dead_code)]
pub(crate) fn establish_connection(
    database_url: &str,
) -> Result<SqliteConnection, diesel::ConnectionError> {
    SqliteConnection::establish(database_url)
}

pub(crate) fn create_pool(database_url: &str) -> Result<SqlitePool, PoolError> {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    Pool::builder().build(manager)
}

pub(crate) fn shared_pool() -> Result<SqlitePool, PoolError> {
    if let Some(pool) = SHARED_POOL.get() {
        return Ok(pool.clone());
    }

    let pool = create_pool(database_url().as_str())?;
    let _ = SHARED_POOL.set(pool);

    Ok(SHARED_POOL
        .get()
        .expect("shared database pool should be initialized")
        .clone())
}
