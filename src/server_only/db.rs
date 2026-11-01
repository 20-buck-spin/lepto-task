use std::{str::FromStr, time::Duration};

use super::error::AppError;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};

fn get_database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./data/app.db?mode=rwc".to_string())
}

pub async fn init_db() -> Result<SqlitePool, AppError> {
    let database_url = get_database_url();

    let connect_options = SqliteConnectOptions::from_str(&database_url)
        .map_err(|e| AppError::DbError(e.to_string()))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(Duration::from_secs(5))
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        // SQLite allows only one writer at a time. Keeping this at 1
        // prevents SQLITE_BUSY storms concurrent writers.
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(10))
        .connect_with(connect_options)
        .await
        .map_err(|e| AppError::DbError(e.to_string()))?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| AppError::DbError(e.to_string()))?;

    Ok(pool)
}

pub async fn get_connection(pool: &SqlitePool) -> Result<sqlx::SqliteConnection, AppError> {
    Ok(pool
        .acquire()
        .await
        .map_err(|e| AppError::DbError(e.to_string()))?
        .detach())
}
