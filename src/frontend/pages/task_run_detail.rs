use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::frontend::components::{
    runs_comps::run_details::TaskRunDetails,
    task_run_logs_comps::task_run_logs_table::TaskRunLogsTable,
};

#[component]
pub fn TaskRunDetail() -> impl IntoView {
    let params = use_params_map();

    let run_id = Memo::new(move |_| {
        params
            .read()
            .get("task_run_id")
            .and_then(|id| id.parse::<i64>().ok())
    });
    view! {
        <div class="page-block">
            {move || {
                match run_id.get() {
                    Some(id) => view! {
                        <div>
                            <TaskRunDetails run_id=id/>
                            <TaskRunLogsTable run_id=id/>
                        </div>
                    }.into_any(),
                    None => view! {
                        <div>
                            <h1>Invalid Task Run</h1>
                            <p>Please provide a valid task run ID.</p>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
