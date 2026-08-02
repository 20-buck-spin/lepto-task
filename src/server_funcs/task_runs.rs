use chrono::{DateTime, Utc};
use leptos::prelude::*;

use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::{query_as, Pool, QueryBuilder, Sqlite};

#[cfg(feature = "ssr")]
use crate::server_only::task_manager::manager::TaskManager;

use crate::models::{
    list_with_count::DataWithCount,
    task_models::{RunStatus, RunningTaskPublic, Task, TaskRun, TaskRunDetailsPublic, TaskRunList},
};

#[cfg(feature = "ssr")]
use crate::ws::ws_context::WsContext;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct TaskRunFilters {
    pub run_id: Option<i64>,
    pub task_id: Option<i64>,
    pub created_dt_from: Option<DateTime<Utc>>,
    pub created_dt_to: Option<DateTime<Utc>>,
    pub statuses: Vec<RunStatus>,
    pub limit: i64,
    pub offset: i64,
}

impl Default for TaskRunFilters {
    fn default() -> Self {
        TaskRunFilters {
            run_id: None,
            created_dt_from: None,
            created_dt_to: None,
            task_id: None,
            statuses: vec![],
            limit: 30,
            offset: 0,
        }
    }
}

#[cfg(feature = "ssr")]
// Helper function to reduce duplication
fn add_filters(query_builder: &mut QueryBuilder<sqlx::Sqlite>, filters: &TaskRunFilters) {
    if let Some(run_id) = filters.run_id {
        query_builder.push("AND id = ");
        query_builder.push_bind(run_id);
    }
    if let Some(task_id) = filters.task_id {
        query_builder.push("AND task_id = ");
        query_builder.push_bind(task_id);
    }

    if let Some(created_dt_from) = filters.created_dt_from {
        query_builder.push("AND created_dt >= ");
        query_builder.push_bind(created_dt_from);
    }

    if let Some(created_dt_to) = filters.created_dt_to {
        query_builder.push("AND created_dt <= ");
        query_builder.push_bind(created_dt_to);
    }

    if !filters.statuses.is_empty() {
        query_builder.push("AND status IN (");
        let mut separated = query_builder.separated(", ");
        for status in &filters.statuses {
            separated.push_bind(status.to_string());
        }
        separated.push_unseparated(") ");
    };
}

#[server]
pub async fn get_task_run(run_id: i64) -> Result<TaskRunDetailsPublic, ServerFnError> {
    let db: Pool<Sqlite> = expect_context();
    let task_manager: TaskManager = expect_context();
    // Check if task is in current running tasks (in memory)

    if let Some(running_task) = task_manager.running_tasks.get(&run_id) {
        let val = running_task.value().read().await;
        return Ok(TaskRunDetailsPublic {
            task: val.task.clone(),
            run_id,
            max_cpu_usage: val.max_cpu_usage,
            current_cpu_usage: val.current_cpu_usage,
            max_ram_usage: val.max_ram_usage,
            current_ram_usage: val.current_ram_usage,
            started_at: val.started_at,
            finish_dt: None,
            status: RunStatus::InProcess,
        });
    };

    let mut task_run_query = sqlx::QueryBuilder::new("SELECT * FROM task_run WHERE id = ");
    task_run_query.push_bind(run_id);
    let task_run = task_run_query
        .build_query_as::<TaskRun>()
        .fetch_one(&db)
        .await?;

    let task = sqlx::query_as!(Task, "SELECT * FROM task WHERE id = ?", task_run.task_id)
        .fetch_one(&db)
        .await?;

    Ok(TaskRunDetailsPublic {
        task,
        run_id,
        max_cpu_usage: task_run.max_cpu_percent,
        current_cpu_usage: 0.0,
        max_ram_usage: task_run.max_ram_bytes,
        current_ram_usage: 0,
        started_at: task_run.created_dt,
        status: task_run.status,
        finish_dt: task_run.finish_dt,
    })
}

#[server]
pub async fn get_task_runs(
    filters: TaskRunFilters,
) -> Result<DataWithCount<TaskRunList>, ServerFnError> {
    let db: Pool<Sqlite> = expect_context();

    // Step 1: Get the task runs with your existing query
    let mut query_builder: QueryBuilder<Sqlite> =
        QueryBuilder::new("SELECT * FROM task_run WHERE 1=1 ");
    add_filters(&mut query_builder, &filters);
    query_builder.push("ORDER BY created_dt DESC ");
    query_builder.push("LIMIT ");
    query_builder.push_bind(filters.limit);
    query_builder.push(" OFFSET ");
    query_builder.push_bind(filters.offset);
    let runs: Vec<TaskRun> = query_builder.build_query_as().fetch_all(&db).await?;

    // Step 2: Get all related tasks
    if runs.is_empty() {
        return Ok(DataWithCount {
            data: vec![],
            total_count: 0,
        });
    }

    let task_ids: Vec<i64> = runs.iter().map(|r| r.task_id).collect();

    // Use a simple IN clause
    let placeholders = task_ids
        .iter()
        .map(|task_id| task_id.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let query = format!("SELECT * FROM task WHERE id IN ({})", placeholders);

    let tasks: Vec<Task> = sqlx::query_as(&query).fetch_all(&db).await?;

    // Create a map for O(1) lookups
    let task_map: std::collections::HashMap<i64, Task> =
        tasks.into_iter().map(|task| (task.id, task)).collect();

    // Combine the results
    let results = runs
        .into_iter()
        .filter_map(|run| {
            task_map.get(&run.task_id).map(|task| TaskRunList {
                run,
                task: task.clone(),
            })
        })
        .collect();

    let mut count_query_builder = QueryBuilder::new("SELECT COUNT (1) FROM task_run WHERE 1=1 ");
    add_filters(&mut count_query_builder, &filters);
    let count: i64 = count_query_builder
        .build_query_scalar()
        .fetch_one(&db)
        .await?;

    Ok(DataWithCount {
        data: results,
        total_count: count,
    })
}

#[server]
pub async fn run_task(task_id: i64) -> Result<(), ServerFnError> {
    let task_manager: TaskManager = expect_context();
    let ws_context: WsContext = expect_context();
    let db: Pool<Sqlite> = expect_context();

    let task: Task = query_as!(Task, "SELECT * FROM task WHERE id = ?", task_id)
        .fetch_one(&db)
        .await?;

    task_manager.run_task(task, db, ws_context).await?;
    Ok(())
}

#[server]
pub async fn cancel_run(run_id: i64) -> Result<(), ServerFnError> {
    let task_manager: TaskManager = expect_context();
    task_manager.cancel_ongoing_task(run_id).await?;
    Ok(())
}

#[server]
pub async fn get_running_tasks() -> Result<Vec<RunningTaskPublic>, ServerFnError> {
    let task_manager: TaskManager = expect_context();
    Ok(task_manager.view_running_tasks().await)
}
