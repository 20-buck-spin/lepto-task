use super::error::AppError;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

fn get_database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./data/app.db?mode=rwc".to_string())
}

pub async fn init_db() -> Result<SqlitePool, AppError> {
    let database_url = get_database_url();
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
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
