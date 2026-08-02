use std::str::FromStr;

use crate::models::list_with_count::DataWithCount;
use crate::models::queued_task::QueuedTask;
use crate::models::task_models::{Task, TaskCreateUpdate, TaskFilters};
use croner::Cron;
use leptos::{logging, prelude::*};

#[cfg(feature = "ssr")]
use sqlx::{Pool, Sqlite};

#[cfg(feature = "ssr")]
use crate::server_only::error::AppError;
#[cfg(feature = "ssr")]
use crate::server_only::task_manager::manager::TaskManager;

#[server]
pub async fn create_update_task(data: TaskCreateUpdate) -> Result<Task, ServerFnError> {
    let db: Pool<Sqlite> = expect_context();
    let task_manager: TaskManager = expect_context();

    let task_cron = Cron::from_str(&data.cron_expr).map_err(|_| AppError::CronExprError)?;

    match data.id {
        // Throw update statement if id exists
        Some(id) => {
            match sqlx::query_as!(
                Task,
                "
                    UPDATE task SET
                    name = ?2,
                    command = ?3,
                    working_dir = ?4,
                    cron_expr = ?5,
                    is_active = ?6
                    WHERE id = ?1
                    RETURNING *
                ",
                id,
                data.name,
                data.command,
                data.working_dir,
                data.cron_expr,
                data.is_active
            )
            .fetch_all(&db)
            .await
            {
                Ok(tasks) => match tasks.into_iter().next() {
                    Some(task) => {
                        if let Err(err) = task_manager.upsert_task(task.clone(), task_cron).await {
                            logging::error!("Failed to upsert task after update. Err {}", err)
                        };
                        return Ok(task);
                    }
                    None => {
                        return Err(ServerFnError::ServerError(
                            "Could not get row from db on update".into(),
                        ))
                    }
                },
                Err(err) => return Err(ServerFnError::ServerError(err.to_string())),
            };
        }

        // Else throw insert satetment
        None => {
            match sqlx::query_as!(
                Task,
                "
                    INSERT INTO task
                    (name, command, working_dir, cron_expr, is_active)
                    VALUES
                    (?1, ?2, ?3, ?4, ?5) RETURNING *
                ",
                data.name,
                data.command,
                data.working_dir,
                data.cron_expr,
                data.is_active
            )
            .fetch_all(&db)
            .await
            {
                Ok(tasks) => match tasks.into_iter().next() {
                    Some(task) => {
                        if let Err(err) = task_manager.upsert_task(task.clone(), task_cron).await {
                            logging::error!("Failed to upsert task after create. Err {}", err)
                        };
                        return Ok(task);
                    }

                    None => {
                        return Err(ServerFnError::ServerError(
                            "Could not get row from db on update".into(),
                        ))
                    }
                },
                Err(err) => return Err(ServerFnError::ServerError(err.to_string())),
            };
        }
    }
}

#[server]
pub async fn get_tasks(filters: TaskFilters) -> Result<DataWithCount<Task>, ServerFnError> {
    let db: Pool<Sqlite> = expect_context();

    let tasks = sqlx::query_as!(
        Task,
        "
            SELECT * FROM task
            LIMIT ?
            OFFSET ?
        ",
        filters.limit,
        filters.offset
    )
    .fetch_all(&db)
    .await?;

    let count: i64 = sqlx::query!(
        "
            SELECT COUNT(1) as total_count FROM task
            LIMIT ?
            OFFSET ?
        ",
        filters.limit,
        filters.offset
    )
    .fetch_one(&db)
    .await?
    .total_count;

    Ok(DataWithCount {
        data: tasks,
        total_count: count,
    })
}

#[server]
pub async fn get_queued_tasks() -> Result<Vec<QueuedTask>, ServerFnError> {
    let manager: TaskManager = expect_context();
    Ok(manager.view_scheduled_task_info().await)
}
