use crate::models::{
    list_with_count::DataWithCount,
    task_models::{RunLog, RunLogFilters},
};
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use sqlx::QueryBuilder;

#[server]
pub async fn get_task_run_logs(
    filters: RunLogFilters,
) -> Result<DataWithCount<RunLog>, ServerFnError> {
    use sqlx::{Pool, QueryBuilder, Sqlite};
    let db: Pool<Sqlite> = expect_context();

    let mut query_builder: QueryBuilder<sqlx::Sqlite> =
        sqlx::QueryBuilder::new("SELECT * FROM runlog WHERE 1=1 ");
    add_filters(&mut query_builder, &filters);
    query_builder.push("LIMIT ");
    query_builder.push_bind(filters.limit);
    query_builder.push(" OFFSET ");
    query_builder.push_bind(filters.offset);

    let run_logs: Vec<RunLog> = query_builder.build_query_as().fetch_all(&db).await?;

    let mut count_query_builder: QueryBuilder<sqlx::Sqlite> =
        sqlx::QueryBuilder::new("SELECT COUNT(*) FROM runlog WHERE 1=1 ");

    add_filters(&mut count_query_builder, &filters);

    let total_count: i64 = count_query_builder
        .build_query_scalar()
        .fetch_one(&db)
        .await?;

    Ok(DataWithCount {
        data: run_logs,
        total_count,
    })
}

#[cfg(feature = "ssr")]
// Helper function to reduce duplication
fn add_filters(query_builder: &mut QueryBuilder<sqlx::Sqlite>, filters: &RunLogFilters) {
    if !filters.levels.is_empty() {
        query_builder.push("AND level in (");
        let mut separated = query_builder.separated(", ");
        for level in &filters.levels {
            separated.push_bind(level.to_string());
        }
        separated.push_unseparated(") ");
    };

    query_builder.push("AND run_id = ");
    query_builder.push_bind(filters.run_id);

    if let Some(created_dt_from) = filters.created_dt_from {
        query_builder.push("AND created_dt >= ");
        query_builder.push_bind(created_dt_from);
    }
    if let Some(created_dt_to) = filters.created_dt_to {
        query_builder.push("AND created_dt <= ");
        query_builder.push_bind(created_dt_to);
    }

    if let Some(query_str) = &filters.query_string {
        query_builder.push(
            "AND id in (
                SELECT rowid FROM runlog_fts
                WHERE run_id = ",
        );
        query_builder.push_bind(filters.run_id);
        query_builder.push(
            "
                AND runlog_fts match ",
        );
        // cloned because is reused
        query_builder.push_bind(query_str.clone());
        query_builder.push(" )");
    }
}
