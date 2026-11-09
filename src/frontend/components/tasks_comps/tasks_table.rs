use leptos::prelude::*;

use crate::{
    frontend::components::{
        empty_table::EmptyTable, table_skeleton::TableSkeleton,
        tasks_comps::create_update_task_modal::CreateUpdateTaskModal,
    },
    models::task_models::{Task, TaskFilters},
    server_funcs::tasks::get_tasks,
};

#[component]
pub fn TasksTable() -> impl IntoView {
    let task_filters = RwSignal::new(TaskFilters {
        limit: 10,
        offset: 0,
    });
    let tasks = Resource::new(move || task_filters.get(), |filters| get_tasks(filters));
    let show_crud_modal: RwSignal<bool> = RwSignal::new(false);
    let selected_task: RwSignal<Option<Task>> = RwSignal::new(None);

    view! {
        <Show when=move || show_crud_modal.get()>
            <CreateUpdateTaskModal
                selected_task=selected_task
                on_close=move || {show_crud_modal.set(false); selected_task.set(None); tasks.refetch()}
                on_success=move || {show_crud_modal.set(false); selected_task.set(None); tasks.refetch()}
            />
        </Show>

        // Tasks table
        <div class="table-header">
            <h3>"Tasks"</h3>
            <button class="create-to-table-plus" on:click= move |_| show_crud_modal.set(true)>Add Task</button>
        </div>
        <Suspense fallback= || view!{<TableSkeleton cols_len=3 rows_len=3/>}>
        <div class="table-outer-container">
            <div class="table-inner-container">
                {
                    move || {
                            match tasks.get() {
                                Some(Ok(tasks_list)) => {
                                    if tasks_list.data.is_empty() {
                                        view! { <EmptyTable display_message="No tasks found".to_string()/> }.into_any()
                                    } else {
                                        view! {
                                            <table>
                                                <thead>
                                                    <tr>
                                                        <th>"ID"</th>
                                                        <th>"Name"</th>
                                                        <th>"Command"</th>
                                                        <th>"WorkDir"</th>
                                                        <th>"Cron Expression"</th>
                                                        <th>"IsActive"</th>
                                                        <th>"Action"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>

                                                <For
                                                    each= move || tasks_list.data.clone()
                                                    key=|task| task.id
                                                    children= move |task| {
                                                        view!{
                                                        <tr>
                                                            <td>{task.id}</td>
                                                            <td>{task.name.clone()}</td>
                                                            <td>{task.command.clone()}</td>
                                                            <td>{task.working_dir.clone()}</td>
                                                            <td>{task.cron_expr.clone()}</td>
                                                            <td>{task.is_active}</td>
                                                            <td>
                                                                <button
                                                                    on:click= move |_| {
                                                                        selected_task.set(Some(task.clone()));
                                                                        show_crud_modal.set(true)
                                                                    }
                                                                >"edit"</button>
                                                            </td>
                                                        </tr>
                                                    }
                                                    }
                                                />
                                                </tbody>
                                            </table>
                                        }.into_any()
                                    }

                                },
                                Some(Err(err)) => view! { <span>"Got error " {err.to_string()}</span> }.into_any(),
                                None => view! { <TableSkeleton rows_len=3 cols_len=5/> }.into_any(),
                            }
                    }
                }
            </div>
        </div>
        </Suspense>
    }
}
