use crate::infrastructure::db::{DbError, SqlitePool};
use diesel::sqlite::SqliteConnection;

pub(crate) async fn run<T, F>(pool: SqlitePool, operation: F) -> Result<T, DbError>
where
    T: Send + 'static,
    F: FnOnce(&mut SqliteConnection) -> Result<T, DbError> + Send + 'static,
{
    tokio::task::spawn_blocking(move || {
        let mut connection = pool.get()?;
        operation(&mut connection)
    })
    .await?
}
