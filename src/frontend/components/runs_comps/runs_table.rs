use leptos::{logging, prelude::*};
use leptos_router::components::A;

use crate::{
    frontend::components::{
        empty_table::EmptyTable, pagination::Paginator, table_skeleton::TableSkeleton,
    },
    server_funcs::task_runs::{get_task_runs, TaskRunFilters},
};

#[component]
pub fn RunsTable() -> impl IntoView {
    let run_filters = RwSignal::new(TaskRunFilters::default());

    let current_page: RwSignal<i64> = RwSignal::new(1);
    let page_size: RwSignal<i64> = RwSignal::new(30);

    let runs = Resource::new(move || run_filters.get(), |filters| get_task_runs(filters));

    let on_page_change = Callback::new(move |(page_size, page_no): (i64, i64)| {
        run_filters.update(|filters| {
            filters.limit = page_size;
            filters.offset = (page_no - 1) * page_size;
        })
    });

    let on_filters_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        match TaskRunFilters::from_event(&ev) {
            Err(err) => {
                logging::error!("Failed to parse new filters. Err {}", err);
                return;
            }
            Ok(new_filters) => {
                // limit and off set are defaulted
                run_filters.set(new_filters);
            }
        }
    };

    view! {


        <div class="filter-form-container">

            <form on:submit=on_filters_submit>

                <div>
                    <label>Created from</label>
                    <input type="datetime-local" name="created_dt_from"/>
                </div>
                <div>
                    <label>Created to</label>
                    <input name="created_dt_to" type="datetime-local"/>
                </div>
                <div>
                    <label>ID</label>
                    <input type="number" name="run_id"/>
                </div>

                <div>
                    <label>Task ID</label>
                    <input type="number" name="task_id"/>
                </div>

                <div class="multi-select-filter">
                    <div>
                        <input type="checkbox" id="status-debug" name="statuses[]" value="InProcess"/>
                        <label for="status-debug"> In Process </label>
                    </div>
                    <div>
                        <input type="checkbox" id="status-info" name="statuses[]" value="Completed"/>
                        <label for="status-info"> Completed </label>
                    </div>
                    <div>
                        <input type="checkbox" id="status-warn" name="statuses[]" value="Failed"/>
                        <label for="status-warn"> Failed </label>
                    </div>
                    <div>
                        <input type="checkbox" id="status-error" name="statuses[]" value="Terminated"/>
                        <label for="status-error"> Terminated </label>
                    </div>
                    <div>
                        <input type="checkbox" id="status-critical" name="statuses[]" value="Cancelled"/>
                        <label for="status-critical"> Cancelled </label>
                    </div>
                </div>


                <input type="submit"/>
            </form>

        </div>

        <div class="table-header">
            <h3>"TaskRuns"</h3>
        </div>
        <Suspense fallback= || view!{<TableSkeleton cols_len=3 rows_len=3/>}>
            <div class="table-outer-container">
                <div class="table-inner-container">
                    {move || {
                        runs.with(|runs_res| {
                            match runs_res {
                                Some(Ok(runs_list)) => {
                                    if runs_list.data.is_empty() {
                                        view! { <EmptyTable display_message="No runs found".to_string()/> }.into_any()
                                    } else {
                                        view!{
                                            <table>
                                                <thead>
                                                    <tr>
                                                        <th>"Run ID"</th>
                                                        <th>"Task Name"</th>
                                                        <th>"Start dt"</th>
                                                        <th>"Status"</th>
                                                        <th>"End dt"</th>
                                                        <th></th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    {
                                                        let runs_list_inner = runs_list.data.clone();
                                                        runs_list_inner.into_iter().map(|run| {
                                                            view!{
                                                                <tr>
                                                                    <td>{run.run.id}</td>
                                                                    <td>{run.task.name.clone()}</td>
                                                                    <td>{run.run.created_dt.format("%Y-%m-%d %H:%M:%S").to_string()}</td>
                                                                    <td>{run.run.status.to_string()}</td>
                                                                    <td>{match run.run.finish_dt{Some(dt)=>dt.format("%Y-%m-%d %H:%M:%S").to_string(), None=>"-".into()}}</td>
                                                                    <td>
                                                                        <A href=format!("/task_runs/{}", run.run.id)>
                                                                        <button>View</button>
                                                                        </A>
                                                                    </td>
                                                                </tr>
                                                            }
                                                        }).collect::<Vec<_>>()
                                                    }
                                                </tbody>
                                            </table>
                                            <Paginator total_count=runs_list.total_count on_change=on_page_change current_page=current_page page_size=page_size/>
                                        }.into_any()
                                    }
                                }
                                Some(Err(err)) => view! { <span>"Got error " {err.to_string()}</span> }.into_any(),
                                None => view! { <TableSkeleton rows_len=3 cols_len=5/> }.into_any(),
                            }
                        })
                    }}
                </div>
            </div>
        </Suspense>
    }
}
