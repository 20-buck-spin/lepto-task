use crate::{
    frontend::components::runs_comps::cancel_run_btn::CancelRunButton,
    models::task_models::{RunStatus, TaskRunDetailsPublic},
    utils::format_bytes,
};
use leptos::prelude::*;

#[component]
pub fn TaskRunDetailsInner(
    task_run_details: TaskRunDetailsPublic,
    on_cancel: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="table-header">
            <h3>"Task details"</h3>
        </div>
        <div class="task-details-inner-container">
            <div class="task-detail-info-container">
                <span class="task-detail-info-header">Run ID</span>
                <span>{task_run_details.run_id}</span>
            </div>
            <div class="task-detail-info-container">
                <span class="task-detail-info-header">Task ID</span>
                <span>{task_run_details.task.id}</span>
            </div>
            <div class="task-detail-info-container">
                <span class="task-detail-info-header">Task name</span>
                <span>{task_run_details.task.name}</span>
            </div>
            <div class="task-detail-info-container">
                <span class="task-detail-info-header">Status</span>
                <span>{task_run_details.status.to_string()}</span>
            </div>
            <div class="task-detail-info-container">
                <span class="task-detail-info-header">Max cpu usage</span>
                <span>{task_run_details.max_cpu_usage.round()}</span>
            </div>
            <div class="task-detail-info-container">
                <span class="task-detail-info-header">Max ram usage</span>
                <span>{format_bytes(task_run_details.max_ram_usage)}</span>
            </div>
            <div class="task-detail-info-container">
                <span class="task-detail-info-header">Current CPU usage</span>
                <span>{task_run_details.current_cpu_usage.round()}</span>
            </div>
            <div class="task-detail-info-container">
                <span class="task-detail-info-header">Current ram usage</span>
                <span>{format_bytes(task_run_details.current_ram_usage)}</span>
            </div>
            {
                (task_run_details.status == RunStatus::InProcess).then(|| {
                    view!{
                        <CancelRunButton run_id=task_run_details.run_id on_cancel=on_cancel/>
                    }
                })
            }
        </div>
    }
}
