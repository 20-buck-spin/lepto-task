use leptos::prelude::*;

use crate::frontend::components::dashboard_comps::{
    dashboard_sys_info::SystemInfo, queued_tasks::QueuedTasks, running_tasks::RunningTasks,
};

#[component]
pub fn Dashboard() -> impl IntoView {
    let update_running_tasks_trigger = RwSignal::new(0);

    view! {
        <div class="page-block">
            <h3>"System info"</h3>
            <SystemInfo/>
        </div>
        <div class="page-block">
            <h3>"Running Tasks"</h3>
            <RunningTasks update_running_tasks_trigger={update_running_tasks_trigger}/>
        </div>
        <div class="page-block">
            <h3>"Queued Tasks"</h3>
            <QueuedTasks update_running_tasks_trigger={update_running_tasks_trigger}/>
        </div>
    }
}
