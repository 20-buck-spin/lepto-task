use chrono::{DateTime, Utc};
use futures::TryFutureExt;
use sqlx::{Pool, Sqlite};

use crate::{models::task_models::RunStatus, server_only::error::AppError};

pub async fn create_task_run(
    db: &Pool<Sqlite>,
    task_id: i64,
    created_dt: &DateTime<Utc>,
) -> Result<i64, AppError> {
    Ok(sqlx::query_scalar!(
        "INSERT INTO task_run
        (task_id, created_dt, status, max_ram_bytes, max_cpu_percent)
        VALUES
        (?1, ?2, 'InProcess', 0 ,0) RETURNING id
        ",
        task_id,
        created_dt
    )
    .fetch_one(db)
    .await
    .map_err(|e| AppError::DbError(format!("Db error on creating task run. Err {}", e)))?
    .ok_or(AppError::DbError("Got empty task run after insert".into()))?)
}

pub async fn cleanup_tasks_db(db: &Pool<Sqlite>) -> Result<(), AppError> {
    sqlx::query!("UPDATE task_run SET status = 'Cancelled' WHERE status = 'InProcess'")
        .execute(db)
        .map_err(|e| AppError::DbError(format!("Failed to cleanup old task runs. Err {}", e)))
        .await?;
    Ok(())
}

pub async fn mark_task_run_completed(
    db: &Pool<Sqlite>,
    run_id: i64,
    final_status: RunStatus,
    max_cpu_percent: f64,
    max_ram_bytes: i64,
) -> Result<(), AppError> {
    let status_str = final_status.to_string();
    let finished_dt = Utc::now();
    sqlx::query!(
        "UPDATE task_run
        SET status = ?1,
            finish_dt = ?2,
            max_ram_bytes = ?3,
            max_cpu_percent = ?4
        WHERE id = ?5",
        status_str,
        finished_dt,
        max_ram_bytes,
        max_cpu_percent,
        run_id
    )
    .execute(db)
    .await
    .map_err(|e| {
        AppError::DbError(format!(
            "Failed to mark task run completed for run {} Err {}",
            run_id, e
        ))
    })?;
    Ok(())
}
