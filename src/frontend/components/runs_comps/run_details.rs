use std::time::Duration;

use crate::{
    frontend::components::runs_comps::{
        live_logs::TaskRunLiveLogs, task_run_details_inner::TaskRunDetailsInner,
    },
    models::task_models::RunStatus,
    server_funcs::task_runs::get_task_run,
};
use leptos::{prelude::*, task::spawn_local};

#[component]
pub fn TaskRunDetails(run_id: i64) -> impl IntoView {
    let task_run_info_rsc = Resource::new(move || run_id, |run_id| get_task_run(run_id));

    // check for task_run_info, if pending, poll the enpoint once every 2 seconds
    let interval_handle = StoredValue::new(Option::<IntervalHandle>::None);
    let is_in_process = RwSignal::new(false);

    let on_cancel = Callback::new(move |()| {
        spawn_local(async move {
            task_run_info_rsc.refetch();
        });
    });

    Effect::new(move || {
        let Some(Ok(task_data)) = task_run_info_rsc.get() else {
            return;
        };
        is_in_process.set(task_data.status == RunStatus::InProcess);
        if task_data.status == RunStatus::InProcess {
            if interval_handle.get_value().is_none() {
                let handle = set_interval_with_handle(
                    move || {
                        task_run_info_rsc.refetch();
                    },
                    Duration::from_secs(2), // Changed to 2 seconds
                )
                .expect("Failed to set handle");

                interval_handle.set_value(Some(handle));
            }
        } else {
            if let Some(handle) = interval_handle.get_value() {
                handle.clear();
                interval_handle.set_value(None);
            }
        }
    });

    view! {
        <Transition fallback=|| view!{<p> Loading </p>}>
            <div class="task-detail-container">
                {move || {
                    task_run_info_rsc.with(|res| {
                        match res {
                            Some(Ok(data)) => view! {
                                <div class="task-run-info">
                                    <TaskRunDetailsInner task_run_details=data.clone() on_cancel={on_cancel}/>
                                </div>
                            }.into_any(),
                            Some(Err(err)) => view! { <p> Got err {err.to_string()} </p> }.into_any(),
                            None => view! { <p> Loading </p> }.into_any()
                        }
                    })
                }}

                <Show when=move || is_in_process.get()>
                    <div class="recent-logs">
                        <TaskRunLiveLogs run_id=run_id/>
                    </div>
                </Show>
            </div>
        </Transition>
    }
}
